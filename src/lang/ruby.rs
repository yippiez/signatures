//! Heuristic, dependency-free Ruby signature extractor.
//!
//! This is a forward line scanner, not a real parser. Ruby delimits blocks with
//! keywords (`def`/`class`/`module`/`if`/`do` … `end`) rather than indentation,
//! so nesting is tracked with a stack of open blocks: declaration blocks
//! (`class`/`module`/`def`) contribute to the reported nesting level, while
//! plain blocks (`if`, `do`, `case`, …) only keep `end` matching balanced.
//!
//! String/char literals and `#` comments are stripped before scanning so that
//! `def`/`class`/`end` tokens appearing inside them are ignored. Heredocs and a
//! few exotic forms are not modeled; that is an accepted trade-off for the
//! zero-dependency goal. The scanner never panics and always walks whole chars.

use super::Language;
use crate::signature::{Kind, Signature};

pub struct RubyLang;

/// One open block on the nesting stack.
struct Frame {
    /// Whether this block is a declaration (`class`/`module`/`def`) and so
    /// counts toward the reported nesting level.
    is_decl: bool,
}

impl Language for RubyLang {
    fn extract(&self, source: &str) -> Vec<Signature> {
        let lines: Vec<&str> = source.lines().collect();
        let mut out = Vec::new();
        let mut stack: Vec<Frame> = Vec::new();
        let mut in_block_comment = false;
        // Open heredoc terminators, in body order (a line may open several).
        let mut heredocs: Vec<String> = Vec::new();
        // Index in `out` of the currently-open top-level (decl-depth 0)
        // declaration whose block we are still inside. Its `span_end` is updated
        // to the last line consumed while it remains open; when the declaration
        // stack returns to depth 0 (its matching `end`), it is finalized.
        let mut open_top: Option<usize> = None;
        let mut i = 0;

        while i < lines.len() {
            let raw = lines[i];
            let line_no = i + 1;

            // Inside a heredoc body: skip every line (so its `def`/`end`/braces are
            // not scanned) until the terminator delimiter appears alone on a line.
            if !heredocs.is_empty() {
                if raw.trim() == heredocs[0] {
                    heredocs.remove(0);
                }
                i += 1;
                continue;
            }

            // `=begin` / `=end` block comments occupy whole lines.
            if in_block_comment {
                if raw.trim_start().starts_with("=end") {
                    in_block_comment = false;
                }
                i += 1;
                continue;
            }
            if raw.trim_start().starts_with("=begin") {
                in_block_comment = true;
                i += 1;
                continue;
            }

            let clean = strip_line(raw);
            let trimmed = clean.trim_start();

            // `__END__` marks the start of the data section; stop scanning.
            if trimmed == "__END__" {
                break;
            }

            let kind = if is_keyword_token(trimmed, "def") {
                Some(Kind::Function)
            } else if is_keyword_token(trimmed, "class") || is_keyword_token(trimmed, "module") {
                Some(Kind::Class)
            } else {
                None
            };

            if let Some(kind) = kind {
                let is_def = kind == Kind::Function;
                let (text, consumed, opens) = gather_decl(&lines, i, is_def);
                let level = decl_depth(&stack);
                let span_end = (i + consumed).min(lines.len());
                out.push(Signature {
                    indent: level,
                    kind,
                    text,
                    line: line_no,
                    span_end,
                    full: None,
                });
                let idx = out.len() - 1;
                if opens {
                    stack.push(Frame { is_decl: true });
                }
                // Apply block bookkeeping for every line we swallowed (the
                // leading `def`/`class`/`module` keyword is ignored by
                // `apply_blocks`, so the frame we just pushed is authoritative).
                for j in i..(i + consumed) {
                    let cj = strip_line(lines[j]);
                    apply_blocks(&mut stack, &cj);
                    detect_heredoc_openers(&cj, &mut heredocs);
                }
                // A top-level decl that opens a body becomes the open block to be
                // finalized when its `end` is reached. One that does not open a
                // body (endless / one-line method) already has span_end == header.
                if level == 0 && opens && decl_depth(&stack) > 0 {
                    open_top = Some(idx);
                }
                i += consumed;
                continue;
            }

            if let Some((text, full)) = constant_sig(trimmed) {
                let level = decl_depth(&stack);
                // Compute span_end by scanning forward to find the last line of
                // the constant's value (handles multi-line `NAME = {\n…\n}`).
                let rhs_clean = {
                    let clean_trimmed = strip_line(trimmed);
                    clean_trimmed
                        .find('=')
                        .map(|eq| clean_trimmed[eq + 1..].to_string())
                        .unwrap_or_default()
                };
                let span_end = constant_span_end(&lines, i, &rhs_clean);
                out.push(Signature {
                    indent: level,
                    kind: Kind::Constant,
                    text,
                    line: line_no,
                    span_end,
                    full,
                });
            }

            apply_blocks(&mut stack, &clean);
            detect_heredoc_openers(&clean, &mut heredocs);
            // Extend / finalize the open top-level block: while still inside it,
            // its span_end advances to this line; once the declaration stack
            // returns to depth 0 (its `end` line), finalize it.
            if let Some(idx) = open_top {
                out[idx].span_end = line_no;
                if decl_depth(&stack) == 0 {
                    open_top = None;
                }
            }
            i += 1;
        }

        out
    }
}

/// Detect heredoc openers on an already-`strip_line`'d line and push their
/// terminator identifiers onto `terms` (in left-to-right body order). Recognizes
/// `<<~ID`, `<<-ID`, `<<ID`, `<<'ID'`, `<<"ID"`. To avoid mistaking the left-shift
/// operator (`a << b`) for a heredoc, a bare (`<<ID`, no `~`/`-`/quote) opener is
/// only accepted when the identifier starts with an uppercase letter or `_`.
fn detect_heredoc_openers(clean: &str, terms: &mut Vec<String>) {
    let b: Vec<char> = clean.chars().collect();
    let n = b.len();
    let mut i = 0;
    while i + 1 < n {
        if b[i] == '<' && b[i + 1] == '<' {
            let mut j = i + 2;
            let squiggly = j < n && (b[j] == '~' || b[j] == '-');
            if squiggly {
                j += 1;
            }
            if j < n && (b[j] == '\'' || b[j] == '"') {
                let q = b[j];
                j += 1;
                let s = j;
                while j < n && b[j] != q {
                    j += 1;
                }
                if j > s {
                    terms.push(b[s..j].iter().collect());
                }
                i = j + 1;
                continue;
            }
            let id_ok = j < n
                && (b[j] == '_'
                    || b[j].is_ascii_uppercase()
                    || (squiggly && b[j].is_alphabetic()));
            if id_ok {
                let s = j;
                while j < n && (b[j] == '_' || b[j].is_alphanumeric()) {
                    j += 1;
                }
                terms.push(b[s..j].iter().collect());
                i = j;
                continue;
            }
        }
        i += 1;
    }
}

/// Number of declaration blocks currently open (the reported nesting level).
fn decl_depth(stack: &[Frame]) -> usize {
    stack.iter().filter(|f| f.is_decl).count()
}

/// True if `line` begins with the bare keyword `kw` (followed by end-of-line or
/// a non-identifier char), e.g. `def `, `class\t`, but not `define` / `classy`.
fn is_keyword_token(line: &str, kw: &str) -> bool {
    if let Some(rest) = line.strip_prefix(kw) {
        match rest.chars().next() {
            None => true,
            Some(c) => !(c == '_' || c.is_alphanumeric()),
        }
    } else {
        false
    }
}

/// Remove `#` comments and blank out string/char-literal interiors so that
/// keyword tokens inside them are not mistaken for code. Operates char by char
/// to stay panic-free and on char boundaries for non-ASCII input.
fn strip_line(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars();
    // Active string delimiter, if inside a string.
    let mut delim: Option<char> = None;

    while let Some(c) = chars.next() {
        match delim {
            None => match c {
                '#' => break,
                '\'' | '"' | '`' => {
                    out.push(c);
                    delim = Some(c);
                }
                _ => out.push(c),
            },
            Some(d) => {
                if c == '\\' {
                    // Escape: drop this char and the next, keeping length-ish.
                    out.push(' ');
                    if chars.next().is_some() {
                        out.push(' ');
                    }
                } else if c == d {
                    out.push(c);
                    delim = None;
                } else {
                    out.push(' ');
                }
            }
        }
    }
    out
}

/// Collect a `def`/`class`/`module` declaration starting at `start`, joining
/// continuation lines until bracket depth returns to zero (and the line does not
/// end with a continuation comma/backslash). Returns the normalized one-line
/// text (body removed), the number of lines consumed, and whether the
/// declaration opens an `end`-terminated block (false for endless methods).
fn gather_decl(lines: &[&str], start: usize, is_def: bool) -> (String, usize, bool) {
    let mut depth: i32 = 0;
    let mut pieces: Vec<String> = Vec::new();
    let mut consumed = 0;
    let mut endless = false;
    let mut done = false;

    let mut k = start;
    while k < lines.len() {
        consumed += 1;
        // `clean` is used for structural scanning (bracket counting, end-detection).
        // `raw` is the original source line, used for display text so that string
        // literal contents (`= "World"`) are preserved verbatim rather than blanked.
        let clean = strip_line(lines[k]);
        let raw = lines[k];

        // `strip_line` is length-preserving (replaces chars with spaces), so the
        // char-index into `clean` also indexes the same position in `raw`.
        let leading = if k == start {
            raw.len() - raw.trim_start().len()
        } else {
            raw.len() - raw.trim_start().len()
        };
        let src_clean: String =
            if k == start { clean.trim_start().to_string() } else { clean.trim().to_string() };
        let src_raw: String =
            if k == start { raw.trim_start().to_string() } else { raw.trim().to_string() };
        let _ = leading; // only used for comment stripping below

        let chars: Vec<char> = src_clean.chars().collect();
        let raw_chars: Vec<char> = src_raw.chars().collect();
        let mut piece = String::new();
        let mut ci = 0;
        while ci < chars.len() {
            let c = chars[ci];
            // Use the raw char for display; `c` (from clean) for structural decisions.
            let display_c = raw_chars.get(ci).copied().unwrap_or(c);
            match c {
                '(' | '[' | '{' => {
                    depth += 1;
                    piece.push(c);
                }
                ')' | ']' | '}' => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    piece.push(c);
                }
                ';' if depth <= 0 => {
                    done = true;
                    break;
                }
                // Endless method: `def foo = expr` / `def foo(x) = expr`.
                '=' if depth <= 0 && is_def => {
                    let next = chars.get(ci + 1).copied();
                    let prev = piece.chars().last();
                    // `=` is part of the method NAME for a setter `name=(v)` or an
                    // assignment operator `[]=(...)` — recognized by a `(` directly
                    // after it (or a `]` directly before) — not an endless body.
                    let part_of_op = next == Some('=')
                        || next == Some('>')
                        || next == Some('~')
                        || next == Some('(')
                        || matches!(prev, Some('<') | Some('>') | Some('!') | Some('=') | Some(']'));
                    if part_of_op {
                        piece.push(display_c);
                        ci += 1;
                        continue;
                    }
                    endless = true;
                    done = true;
                    break;
                }
                _ => piece.push(display_c),
            }
            ci += 1;
        }

        let piece = piece.trim_end().to_string();
        if !piece.is_empty() {
            pieces.push(piece);
        }

        let tail = src_clean.trim_end();
        let cont = depth > 0 || tail.ends_with(',') || tail.ends_with('\\');
        if done || !cont || k - start > 100 {
            break;
        }
        k += 1;
    }

    let text = tidy(&collapse_ws(&pieces.join(" ")));
    let opens = if is_def { !endless } else { true };
    (text, consumed, opens)
}

/// If `trimmed` is a constant assignment (`NAME = value`), return its normalized
/// signature `NAME = …` plus the full collapsed declaration (for `--output full`).
/// A constant is an identifier beginning with an uppercase letter; the value is
/// elided in the `text`. Comparisons (`==`), hash rockets (`=>`), match (`=~`),
/// augmented assignments and attribute writes (`Foo.bar = …`) are rejected.
///
/// Returns `(truncated_text, full_text)` where `full_text` is `Some` when a
/// non-empty RHS was captured, and `None` when the RHS is empty or absent.
fn constant_sig(trimmed: &str) -> Option<(String, Option<String>)> {
    let mut chars = trimmed.char_indices();
    let (_, first) = chars.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    // Walk the identifier (ASCII letters/digits/underscore).
    let mut end = trimmed.len();
    for (idx, c) in trimmed.char_indices() {
        if idx == 0 {
            continue;
        }
        if c == '_' || c.is_ascii_alphanumeric() {
            continue;
        }
        end = idx;
        break;
    }
    let name = &trimmed[..end];
    let after = trimmed[end..].trim_start();

    if !after.starts_with('=')
        || after.starts_with("==")
        || after.starts_with("=>")
        || after.starts_with("=~")
    {
        return None;
    }

    let truncated = format!("{name} = …");

    // Capture the RHS (everything after the `=` sign) for `--output full`.
    // Strip the leading `=` and any surrounding whitespace, then collapse runs
    // of whitespace to a single space so multi-line-continued values look clean.
    let rhs_raw = after[1..].trim();
    let full = if rhs_raw.is_empty() {
        None
    } else {
        let collapsed = collapse_ws(rhs_raw);
        Some(format!("{name} = {collapsed}"))
    };

    Some((truncated, full))
}

/// Scan forward from `start` (0-indexed) to find the last line of a Ruby constant
/// value that may span multiple lines (e.g. `NAME = {\n  …\n}`). Uses bracket
/// counting on the stripped lines. Returns 1-based line number of the last line.
fn constant_span_end(lines: &[&str], start: usize, rhs_clean: &str) -> usize {
    // Count initial bracket depth from the RHS portion of the first line.
    let mut depth: i32 = 0;
    for c in rhs_clean.chars() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
    }
    if depth <= 0 {
        return start + 1; // 1-based, single line
    }
    // Continue scanning subsequent lines until depth returns to 0.
    let mut k = start + 1;
    while k < lines.len() && k < start + 500 {
        let clean = strip_line(lines[k]);
        for c in clean.chars() {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth <= 0 {
                        return k + 1; // 1-based
                    }
                }
                _ => {}
            }
        }
        k += 1;
    }
    start + 1 // fallback: single line
}

/// Update the block stack for one cleaned line: push for block-opening keywords,
/// pop for `end`. `class`/`module`/`def` are intentionally ignored here because
/// their frames are managed by the caller.
fn apply_blocks(stack: &mut Vec<Frame>, clean: &str) {
    let spaced = clean.replace(';', " ; ");
    let mut at_stmt_start = true;
    for tok in spaced.split_whitespace() {
        if tok == ";" {
            at_stmt_start = true;
            continue;
        }
        let word = leading_ident(tok);
        match word {
            "do" | "begin" | "case" => stack.push(Frame { is_decl: false }),
            "if" | "unless" | "while" | "until" | "for" => {
                if at_stmt_start {
                    stack.push(Frame { is_decl: false });
                }
            }
            "end" => {
                stack.pop();
            }
            _ => {}
        }
        at_stmt_start = false;
    }
}

/// Leading run of identifier characters of a token (for keyword matching).
fn leading_ident(tok: &str) -> &str {
    let mut end = tok.len();
    for (i, c) in tok.char_indices() {
        if c == '_' || c.is_alphanumeric() {
            continue;
        }
        end = i;
        break;
    }
    &tok[..end]
}

/// Collapse every run of whitespace down to a single space and trim the ends.
fn collapse_ws(s: &str) -> String {
    let mut out = String::new();
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// Tidy spacing introduced when joining a multi-line signature: remove spaces
/// hugging brackets/commas and drop trailing commas before a closer.
fn tidy(s: &str) -> String {
    let mut t = s.to_string();
    for (from, to) in [
        ("( ", "("),
        (" )", ")"),
        ("[ ", "["),
        (" ]", "]"),
        ("{ ", "{"),
        (" }", "}"),
        (" ,", ","),
        (", )", ")"),
        (",)", ")"),
        (",]", "]"),
        (",}", "}"),
    ] {
        t = t.replace(from, to);
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sigs(src: &str) -> Vec<Signature> {
        RubyLang.extract(src)
    }

    #[test]
    fn top_level_methods() {
        let s = sigs("def foo(a, b)\n  a + b\nend\n\ndef bar\n  1\nend\n");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].kind, Kind::Function);
        assert_eq!(s[0].text, "def foo(a, b)");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[0].line, 1);
        assert_eq!(s[1].text, "def bar");
        assert_eq!(s[1].indent, 0);
        assert_eq!(s[1].line, 5);
    }

    #[test]
    fn class_with_methods_nested() {
        let src = "class Foo < Base\n  def initialize\n    @x = 1\n  end\n  def run(x)\n    x\n  end\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[0].text, "class Foo < Base");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[1].text, "def initialize");
        assert_eq!(s[1].indent, 1);
        assert_eq!(s[2].text, "def run(x)");
        assert_eq!(s[2].indent, 1);
    }

    #[test]
    fn module_and_deeper_nesting() {
        let src = "module M\n  class C\n    def m\n    end\n  end\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[0].text, "module M");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[1].text, "class C");
        assert_eq!(s[1].indent, 1);
        assert_eq!(s[2].text, "def m");
        assert_eq!(s[2].indent, 2);
    }

    #[test]
    fn multiline_signature_joined() {
        let src = "def foo(\n  a,\n  b,\n  c\n)\n  a\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "def foo(a, b, c)");
        assert_eq!(s[0].indent, 0);
    }

    #[test]
    fn setter_and_operator_methods() {
        // `def name=(v)` / `def []=(k, v)` are not endless methods; they must keep
        // their `=` and args and still push a frame (so siblings nest correctly).
        let src = "class W\n  def name=(val)\n  end\n  def []=(k, v)\n  end\n  def after\n  end\nend\n";
        let s = sigs(src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class W", "def name=(val)", "def []=(k, v)", "def after"]);
        assert!(s.iter().skip(1).all(|x| x.indent == 1));
    }

    #[test]
    fn heredoc_body_ignored() {
        let src = "class O\n  def foo\n    x = <<~SQL\n      def fake\n      end\n    SQL\n  end\n  def bar\n  end\nend\n";
        let s = sigs(src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class O", "def foo", "def bar"]);
        assert_eq!(s[2].indent, 1);
    }

    #[test]
    fn one_line_and_endless_methods() {
        let src = "def a; 1; end\ndef b = 2\ndef c\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].text, "def a");
        assert_eq!(s[1].text, "def b");
        assert_eq!(s[2].text, "def c");
        // None of these should leak nesting onto their siblings.
        assert!(s.iter().all(|x| x.indent == 0));
    }

    #[test]
    fn constants_detected_and_rejected() {
        let src = "MAX_SIZE = 10\nName = \"x\"\nlowercase = 1\nRESULT == other\nFoo.bar = 3\n";
        let texts: Vec<String> = sigs(src)
            .iter()
            .filter(|x| x.kind == Kind::Constant)
            .map(|x| x.text.clone())
            .collect();
        assert_eq!(texts, vec!["MAX_SIZE = …".to_string(), "Name = …".to_string()]);
    }

    #[test]
    fn constants_full_field_captures_rhs() {
        // Numeric literal.
        let s = sigs("MAX_SIZE = 10\n");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "MAX_SIZE = …");
        assert_eq!(s[0].full, Some("MAX_SIZE = 10".to_string()));

        // String literal (strip_line blanks the interior, but the `=` and the
        // quoted delimiters remain so we still capture something).
        let s2 = sigs("GREETING = \"hello\"\n");
        assert_eq!(s2.len(), 1);
        assert_eq!(s2[0].text, "GREETING = …");
        // The RHS captured from strip_line is `"     "` (blanked interior), which
        // after trim is `""`, so full should be non-None (contains the delimiters).
        assert!(s2[0].full.is_some());
        // The full text must start with the constant name.
        assert!(s2[0].full.as_deref().unwrap().starts_with("GREETING = "));

        // Empty RHS (value on next line / comment strips it): full stays None.
        let s3 = sigs("WEIRD = # comment\n");
        assert_eq!(s3.len(), 1);
        assert_eq!(s3[0].text, "WEIRD = …");
        assert_eq!(s3[0].full, None);

        // Array / method-call value.
        let s4 = sigs("FLAGS = [1, 2, 3].freeze\n");
        assert_eq!(s4.len(), 1);
        assert_eq!(s4[0].full, Some("FLAGS = [1, 2, 3].freeze".to_string()));
    }

    #[test]
    fn decl_in_comment_ignored() {
        let src = "def real\n  # def fake\n  x = \"class Nope\"\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "def real");
    }

    #[test]
    fn block_keywords_keep_nesting_balanced() {
        let src = "class A\n  def m\n    [1, 2].each do |i|\n      if i > 0\n        i\n      end\n    end\n  end\n  def n\n  end\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 3);
        assert_eq!(s[2].text, "def n");
        // `n` is a sibling of `m`, still at nesting level 1 despite the
        // intervening `do`/`if`/`end` blocks.
        assert_eq!(s[2].indent, 1);
    }
}
