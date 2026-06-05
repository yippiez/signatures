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
        let mut i = 0;

        while i < lines.len() {
            let raw = lines[i];
            let line_no = i + 1;

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
                out.push(Signature { indent: level, kind, text, line: line_no });
                if opens {
                    stack.push(Frame { is_decl: true });
                }
                // Apply block bookkeeping for every line we swallowed (the
                // leading `def`/`class`/`module` keyword is ignored by
                // `apply_blocks`, so the frame we just pushed is authoritative).
                for j in i..(i + consumed) {
                    apply_blocks(&mut stack, &strip_line(lines[j]));
                }
                i += consumed;
                continue;
            }

            if let Some(text) = constant_sig(trimmed) {
                let level = decl_depth(&stack);
                out.push(Signature { indent: level, kind: Kind::Constant, text, line: line_no });
            }

            apply_blocks(&mut stack, &clean);
            i += 1;
        }

        out
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
        let clean = strip_line(lines[k]);
        let src: String =
            if k == start { clean.trim_start().to_string() } else { clean.trim().to_string() };

        let chars: Vec<char> = src.chars().collect();
        let mut piece = String::new();
        let mut ci = 0;
        while ci < chars.len() {
            let c = chars[ci];
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
                    let part_of_op = next == Some('=')
                        || next == Some('>')
                        || next == Some('~')
                        || matches!(prev, Some('<') | Some('>') | Some('!') | Some('='));
                    if part_of_op {
                        piece.push(c);
                        ci += 1;
                        continue;
                    }
                    endless = true;
                    done = true;
                    break;
                }
                _ => piece.push(c),
            }
            ci += 1;
        }

        let piece = piece.trim_end().to_string();
        if !piece.is_empty() {
            pieces.push(piece);
        }

        let tail = src.trim_end();
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
/// signature `NAME = …`. A constant is an identifier beginning with an uppercase
/// letter; the value is elided. Comparisons (`==`), hash rockets (`=>`), match
/// (`=~`), augmented assignments and attribute writes (`Foo.bar = …`) are
/// rejected.
fn constant_sig(trimmed: &str) -> Option<String> {
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

    Some(format!("{name} = …"))
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
