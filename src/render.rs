//! Turn extracted signatures into colorized, indented output text.

use crate::cli::{Format, OutputMode};
use crate::color::Colors;
use crate::signature::{Kind, Signature};

/// Pick the text to render for a signature honoring `--output`.
fn chosen_text<'a>(s: &'a Signature, output: OutputMode) -> &'a str {
    match output {
        OutputMode::Truncated => &s.text,
        // `full` falls back to `text` when nothing was elided (full == None).
        OutputMode::Full => s.full.as_deref().unwrap_or(&s.text),
    }
}

/// Render exactly ONE signature (no trailing newline) to a String, honoring the
/// chosen format and output mode. Used by both the buffered and streaming paths.
pub fn render_one(
    path: &str,
    s: &Signature,
    colors: &Colors,
    format: Format,
    output: OutputMode,
) -> String {
    let text = chosen_text(s, output);
    match format {
        Format::Plain => {
            let mut line = String::new();
            for _ in 0..s.indent {
                line.push_str("  ");
            }
            line.push_str(&colorize(s.kind, text, colors));
            line
        }
        Format::Jsonl => jsonl_record(path, s.line, s.indent, s.kind, text, colors),
    }
}

/// Build one compact JSON-Lines record. When `colors.enabled` (a TTY), the
/// structural punctuation and keys are dimmed so the values stand out; the
/// values themselves keep the default color. When disabled (piped or
/// `--no-color`) the result is plain, valid JSON. `text` is emitted verbatim, so
/// callers pass an already-cleaned single-line string.
fn jsonl_record(
    path: &str,
    line: usize,
    indent: usize,
    kind: Kind,
    text: &str,
    colors: &Colors,
) -> String {
    let kind_s = match kind {
        Kind::Function => "function",
        Kind::Class => "class",
        Kind::Constant => "constant",
    };
    let d = |s: &str| colors.dim(s);
    let mut o = String::new();
    o.push_str(&d("{\"file\":"));
    o.push_str(&json_quoted(path));
    o.push_str(&d(",\"line\":"));
    o.push_str(&line.to_string());
    o.push_str(&d(",\"indent\":"));
    o.push_str(&indent.to_string());
    o.push_str(&d(",\"kind\":"));
    o.push_str(&json_quoted(kind_s));
    o.push_str(&d(",\"text\":"));
    o.push_str(&json_quoted(text));
    o.push_str(&d("}"));
    o
}

/// `s` as a JSON string literal (with surrounding quotes), returned as a String.
fn json_quoted(s: &str) -> String {
    let mut o = String::new();
    json_string(s, &mut o);
    o
}

/// Collapse every run of whitespace (including newlines) to a single space and
/// trim the ends — turns a multi-line body into one tidy line for jsonl.
fn collapse_ws(s: &str) -> String {
    let mut out = String::new();
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !out.is_empty() && !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    while out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Color one verbatim source line that begins a declaration: preserve its
/// leading indentation, then colorize the rest as a signature of `kind` (leading
/// keywords highlighted, the name colored, the remainder dimmed). When color is
/// disabled this returns the line byte-for-byte.
fn color_source_line(raw: &str, kind: Kind, colors: &Colors) -> String {
    let indent_len = raw.len() - raw.trim_start().len();
    let (indent, rest) = raw.split_at(indent_len);
    format!("{indent}{}", colorize(kind, rest, colors))
}

/// Map each declaration's 1-based start line to its kind, so full mode can color
/// the header lines inside a printed block (the decl itself and any nested
/// declarations whose headers fall within the span).
fn line_kinds(sigs: &[Signature]) -> std::collections::HashMap<usize, Kind> {
    let mut m = std::collections::HashMap::new();
    for s in sigs {
        m.insert(s.line, s.kind);
    }
    m
}

/// Append the rendered block for one file (optional header + its signatures) to
/// `out`. Plain format only emits the header; jsonl never does.
pub fn render(
    path: &str,
    sigs: &[Signature],
    colors: &Colors,
    show_header: bool,
    format: Format,
    output: OutputMode,
    out: &mut String,
) {
    if format == Format::Plain && show_header {
        out.push_str(&colors.header(path));
        out.push('\n');
    }
    for s in sigs {
        out.push_str(&render_one(path, s, colors, format, output));
        out.push('\n');
    }
}

/// Whether each signature should be EMITTED in full mode, by coverage: walking
/// in source order, a declaration is emitted iff its start line is not already
/// inside a previously-emitted declaration's printed span. This yields the
/// outermost declarations plus any declaration whose enclosing block is itself
/// not an emitted signature (e.g. a Rust `impl` block's methods) — each with its
/// full body — while never duplicating members that already appear inside a
/// printed class/struct block.
pub fn full_mode_emit(sigs: &[Signature]) -> Vec<bool> {
    let mut emit = vec![false; sigs.len()];
    let mut covered_until = 0usize; // 1-based last line covered by an emitted span
    for (i, s) in sigs.iter().enumerate() {
        if s.line > covered_until {
            emit[i] = true;
            covered_until = covered_until.max(s.span_end.max(s.line));
        }
    }
    emit
}

/// Render ONE declaration in full mode. Plain format returns the verbatim source
/// span `s.line..=s.span_end`, with each line that begins a declaration colored
/// like the outline (header keywords + name highlighted) and body lines left
/// as-is — `lk` maps decl-start lines to their kind. Jsonl returns one record
/// whose `text` is the body collapsed to a single tidy line (no embedded
/// newlines), optionally colored. The CALLER decides which signatures to render
/// (see [`full_mode_emit`]); this maps a chosen signature to its span. Returns
/// `None` only if the span is out of range.
pub fn render_one_full(
    path: &str,
    s: &Signature,
    lines: &[&str],
    format: Format,
    colors: &Colors,
    lk: &std::collections::HashMap<usize, Kind>,
) -> Option<String> {
    // Clamp the 1-based span to the available lines; `line`/`span_end` come from
    // the same source so they are normally in range.
    let start = s.line.saturating_sub(1);
    let end = s.span_end.max(s.line).min(lines.len());
    if start >= lines.len() {
        return None;
    }
    match format {
        Format::Plain => {
            let mut block = String::new();
            for (off, raw) in lines[start..end].iter().enumerate() {
                if off > 0 {
                    block.push('\n');
                }
                let lineno = start + off + 1;
                match lk.get(&lineno) {
                    Some(kind) => block.push_str(&color_source_line(raw, *kind, colors)),
                    None => block.push_str(raw),
                }
            }
            Some(block)
        }
        Format::Jsonl => {
            let body = collapse_ws(&lines[start..end].join("\n"));
            Some(jsonl_record(path, s.line, s.indent, s.kind, &body, colors))
        }
    }
}

/// Append the full-mode rendered block for one file to `out`. Emits the
/// outermost declarations (see [`full_mode_emit`]); members already inside a
/// printed block are not repeated. Plain format prints each decl's source span
/// (header lines colored) with exactly one blank line between consecutive decls
/// (and a header line first when `show_header`). Jsonl prints one record per
/// emitted decl. `lines` is the original source split into lines.
pub fn render_full(
    path: &str,
    sigs: &[Signature],
    lines: &[&str],
    colors: &Colors,
    show_header: bool,
    format: Format,
    out: &mut String,
) {
    if format == Format::Plain && show_header {
        out.push_str(&colors.header(path));
        out.push('\n');
    }
    let emit = full_mode_emit(sigs);
    let lk = line_kinds(sigs);
    let mut first = true;
    for (s, &keep) in sigs.iter().zip(emit.iter()) {
        if !keep {
            continue;
        }
        let block = match render_one_full(path, s, lines, format, colors, &lk) {
            Some(b) => b,
            None => continue,
        };
        if format == Format::Plain && !first {
            // Exactly one blank line between consecutive emitted decls.
            out.push('\n');
        }
        out.push_str(&block);
        out.push('\n');
        first = false;
    }
}

/// Build the decl-start-line -> kind map for full-mode header coloring. Exposed
/// so the streaming path in `main` can reuse it per file.
pub fn full_line_kinds(sigs: &[Signature]) -> std::collections::HashMap<usize, Kind> {
    line_kinds(sigs)
}

/// Append `s` to `out` as a JSON string literal (including the surrounding
/// quotes). Hand-rolled, no dependencies. Escapes `\`, `"`, control chars
/// (`\n \r \t` and `\u00XX` for other bytes < 0x20). UTF-8 is kept as-is.
pub fn json_string(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Keyword tokens highlighted as keywords across all supported languages
/// (declaration keywords plus common modifiers). Anything not in this set that
/// precedes a name — e.g. a C/Java return type — is left uncolored.
const KEYWORDS: &[&str] = &[
    // declaration keywords
    "def", "async", "fn", "func", "fun", "function", "class", "struct", "enum", "trait", "union",
    "type", "interface", "record", "namespace", "typedef", "impl", "const", "constexpr", "let",
    "val", "var", "static", "object", "mixin", "extension", "protocol", "typealias", "delegate",
    "actor", "sub", "method", "proc", "module",
    // modifiers
    "pub", "public", "private", "protected", "abstract", "export", "default", "override",
    "virtual", "inline", "synchronized", "native", "transient", "volatile", "strictfp", "unsafe",
    "extern", "final", "mut", "internal", "sealed", "partial", "readonly", "data", "open",
    "suspend", "operator", "companion", "lateinit", "infix", "tailrec", "template", "case",
    "factory", "covariant", "late", "fileprivate", "convenience", "required", "mutating",
    "local", "declare", "typeset",
];

fn is_keyword(w: &str) -> bool {
    KEYWORDS.contains(&w) || w.starts_with("pub(")
}

/// Color a single signature's tokens according to its kind.
fn colorize(kind: Kind, text: &str, c: &Colors) -> String {
    match kind {
        Kind::Function => color_callable(text, c),
        Kind::Class => color_class(text, c),
        Kind::Constant => {
            let (name, tail) = split_name(text);
            format!("{}{}", c.constant(name), c.dim(tail))
        }
    }
}

/// Color a function/method: highlight leading keywords, color the name (the
/// identifier just before the parameter list) and dim the rest. Works for
/// Python `def`, Rust `fn`, Go `func`, JS/TS, Java and C alike.
fn color_callable(text: &str, c: &Colors) -> String {
    let paren = match text.find('(') {
        Some(i) => i,
        None => return color_leading_keyword(text, c),
    };
    let before = &text[..paren];
    let name = trailing_ident(before);
    if name.is_empty() {
        return color_leading_keyword(text, c);
    }
    let prefix = &before[..before.len() - name.len()];
    let tail = &text[paren..];

    // Preserve the prefix's exact spacing/punctuation (e.g. `Greeter.` in a Lua
    // method) while coloring keyword words within it — don't re-join on spaces.
    let mut out = color_prefix_keywords(prefix, c);
    out.push_str(&c.name(name));
    out.push_str(&c.dim(tail));
    out
}

/// Color any keyword words inside `prefix`, leaving every other character
/// (spaces, dots, `::`, etc.) exactly as-is.
fn color_prefix_keywords(prefix: &str, c: &Colors) -> String {
    let mut out = String::new();
    let mut word = String::new();
    for ch in prefix.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            if !word.is_empty() {
                out.push_str(&if is_keyword(&word) { c.kw(&word) } else { word.clone() });
                word.clear();
            }
            out.push(ch);
        }
    }
    if !word.is_empty() {
        out.push_str(&if is_keyword(&word) { c.kw(&word) } else { word });
    }
    out
}

/// Color a type/class declaration: highlight modifier/type keywords, color the
/// name following the type keyword, dim the rest.
fn color_class(text: &str, c: &Colors) -> String {
    let mut out = String::new();
    let mut rest = text.trim_start();
    loop {
        let (w, after) = take_word(rest);
        if w.is_empty() {
            out.push_str(rest);
            return out;
        }
        if is_class_keyword(w) {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&c.kw(w));
            let r = after.trim_start();
            let (name, tail) = split_name(r);
            out.push(' ');
            out.push_str(&c.name(name));
            out.push_str(&c.dim(tail));
            return out;
        }
        // A leading modifier keyword.
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&if is_keyword(w) { c.kw(w) } else { w.to_string() });
        rest = after.trim_start();
    }
}

fn is_class_keyword(w: &str) -> bool {
    matches!(
        w,
        "class"
            | "struct"
            | "enum"
            | "trait"
            | "union"
            | "type"
            | "interface"
            | "record"
            | "namespace"
            | "typedef"
            | "object"
            | "mixin"
            | "extension"
            | "protocol"
            | "typealias"
            | "delegate"
            | "actor"
            | "module"
    )
}

/// Color only a leading keyword if present; otherwise return the text as-is.
fn color_leading_keyword(text: &str, c: &Colors) -> String {
    let (w, after) = take_word(text.trim_start());
    if is_keyword(w) {
        format!("{} {}", c.kw(w), after.trim_start())
    } else {
        text.to_string()
    }
}

/// Split off a leading whitespace-delimited word.
fn take_word(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    let end = s.find(char::is_whitespace).unwrap_or(s.len());
    s.split_at(end)
}

/// Trailing run of identifier characters in `s`. Unicode-aware: walks back over
/// whole chars so multi-byte identifiers (e.g. `naïve`) are not split mid-char.
fn trailing_ident(s: &str) -> &str {
    let mut idx = s.len();
    for (i, ch) in s.char_indices().rev() {
        if ch == '_' || ch == '$' || ch.is_alphanumeric() {
            idx = i;
        } else {
            break;
        }
    }
    &s[idx..]
}

/// Split a declaration remainder into its leading identifier and the rest
/// (parens, bases, `= …`, etc.).
fn split_name(s: &str) -> (&str, &str) {
    let end = s
        .find(|ch| matches!(ch, '(' | ':' | ' ' | '['))
        .unwrap_or(s.len());
    s.split_at(end)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(kind: Kind, text: &str) -> Signature {
        Signature { indent: 0, kind, text: text.to_string(), line: 1, span_end: 1, full: None }
    }

    #[test]
    fn no_color_has_no_escapes() {
        let colors = Colors { enabled: false };
        let mut out = String::new();
        render(
            "a.py",
            &[
                sig(Kind::Class, "class Foo(Base):"),
                sig(Kind::Function, "def run(self):"),
                sig(Kind::Constant, "MAX = …"),
            ],
            &colors,
            true,
            Format::Plain,
            OutputMode::Truncated,
            &mut out,
        );
        assert!(!out.contains('\x1b'), "expected no ANSI escapes: {out:?}");
        assert!(out.contains("class Foo(Base):"));
        assert!(out.contains("MAX = …"));
    }

    #[test]
    fn color_wraps_tokens() {
        let colors = Colors { enabled: true };
        let mut out = String::new();
        render(
            "a.py",
            &[sig(Kind::Function, "def run(self):")],
            &colors,
            false,
            Format::Plain,
            OutputMode::Truncated,
            &mut out,
        );
        assert!(out.contains("\x1b[1;35mdef\x1b[0m"));
        assert!(out.contains("\x1b[1;36mrun\x1b[0m"));
    }

    #[test]
    fn nesting_indents() {
        let colors = Colors { enabled: false };
        let mut out = String::new();
        let mut s = sig(Kind::Function, "def m(self):");
        s.indent = 1;
        render(
            "a.py",
            &[s],
            &colors,
            false,
            Format::Plain,
            OutputMode::Truncated,
            &mut out,
        );
        assert!(out.starts_with("  def m(self):"));
    }

    #[test]
    fn json_escapes() {
        let mut o = String::new();
        json_string("a\"b\\c\nd\te\r\u{1}f", &mut o);
        assert_eq!(o, "\"a\\\"b\\\\c\\nd\\te\\r\\u0001f\"");
    }

    #[test]
    fn json_keeps_utf8() {
        let mut o = String::new();
        json_string("MAX = …", &mut o);
        assert_eq!(o, "\"MAX = …\"");
    }

    #[test]
    fn jsonl_record_shape() {
        // Disabled (piped / --no-color): plain, valid, byte-stable JSON.
        let s = sig(Kind::Constant, "MAX = …");
        let line = render_one("a.py", &s, &Colors { enabled: false }, Format::Jsonl, OutputMode::Truncated);
        assert!(!line.contains('\x1b'), "piped jsonl must not be colorized: {line:?}");
        assert_eq!(
            line,
            "{\"file\":\"a.py\",\"line\":1,\"indent\":0,\"kind\":\"constant\",\"text\":\"MAX = …\"}"
        );
        // Enabled (a TTY): structural punctuation is dimmed so values stand out.
        let colored = render_one("a.py", &s, &Colors { enabled: true }, Format::Jsonl, OutputMode::Truncated);
        assert!(colored.contains('\x1b'), "jsonl on a tty should be colorized: {colored:?}");
    }

    // ---- full mode ----

    fn span_sig(kind: Kind, text: &str, line: usize, span_end: usize, indent: usize) -> Signature {
        Signature { indent, kind, text: text.to_string(), line, span_end, full: None }
    }

    #[test]
    fn full_prints_verbatim_span() {
        // A function whose body spans lines 1..=3 prints those raw lines.
        let src = "def greet(name):\n    \"\"\"doc\"\"\"\n    return name\n";
        let lines: Vec<&str> = src.lines().collect();
        let s = span_sig(Kind::Function, "def greet(name):", 1, 3, 0);
        let lk = std::collections::HashMap::new();
        let block =
            render_one_full("a.py", &s, &lines, Format::Plain, &Colors { enabled: false }, &lk)
                .unwrap();
        assert_eq!(block, "def greet(name):\n    \"\"\"doc\"\"\"\n    return name");
    }

    #[test]
    fn full_emit_covers_members_but_not_uncovered() {
        // Coverage rule: a member inside a printed class span is NOT re-emitted,
        // but a declaration whose enclosing block is itself not a signature
        // (e.g. a Rust `impl` block's method) IS emitted on its own.
        let sigs = vec![
            span_sig(Kind::Class, "class C:", 1, 3, 0), // span 1..=3
            span_sig(Kind::Function, "def m(self):", 2, 3, 1), // inside C -> covered
            // A method at line 5 inside a non-emitted `impl`-like wrapper: its
            // start line (5) is past the class span (3), so it is emitted.
            span_sig(Kind::Function, "fn new()", 5, 7, 1),
        ];
        let emit = full_mode_emit(&sigs);
        assert_eq!(emit, vec![true, false, true]);
    }

    #[test]
    fn full_one_blank_between_decls_and_class_includes_members() {
        // Two top-level decls: a constant (line 1) and a class spanning 3..=5
        // whose method body appears inside the class block, NOT as a separate
        // emitted entry.
        let src = "MAX = 5\n\nclass C:\n    def m(self):\n        pass\n";
        let lines: Vec<&str> = src.lines().collect();
        let sigs = vec![
            span_sig(Kind::Constant, "MAX = …", 1, 1, 0),
            span_sig(Kind::Class, "class C:", 3, 5, 0),
            span_sig(Kind::Function, "def m(self):", 4, 5, 1), // nested, skipped
        ];
        let mut out = String::new();
        render_full("a.py", &sigs, &lines, &Colors { enabled: false }, false, Format::Plain, &mut out);
        assert_eq!(out, "MAX = 5\n\nclass C:\n    def m(self):\n        pass\n");
        // The method line is present (inside the class body) but not repeated.
        assert_eq!(out.matches("def m(self):").count(), 1);
    }

    #[test]
    fn full_colors_header_line_only_when_enabled() {
        let src = "def f():\n    return 1\n";
        let lines: Vec<&str> = src.lines().collect();
        let sigs = vec![span_sig(Kind::Function, "def f():", 1, 2, 0)];
        // Disabled: verbatim, no escapes.
        let mut plain = String::new();
        render_full("a.py", &sigs, &lines, &Colors { enabled: false }, false, Format::Plain, &mut plain);
        assert!(!plain.contains('\x1b'));
        assert_eq!(plain, "def f():\n    return 1\n");
        // Enabled: the declaration header line is colored, the body line is not.
        let mut col = String::new();
        render_full("a.py", &sigs, &lines, &Colors { enabled: true }, false, Format::Plain, &mut col);
        assert!(col.contains("\x1b[1;35mdef\x1b[0m"), "def keyword should be magenta: {col:?}");
        assert!(col.contains("    return 1"), "body line stays verbatim: {col:?}");
    }

    #[test]
    fn full_jsonl_collapses_body_to_one_line() {
        let src = "def f(a):\n    return a\n";
        let lines: Vec<&str> = src.lines().collect();
        let s = span_sig(Kind::Function, "def f(a):", 1, 2, 0);
        let lk = std::collections::HashMap::new();
        let line =
            render_one_full("a.py", &s, &lines, Format::Jsonl, &Colors { enabled: false }, &lk)
                .unwrap();
        assert!(!line.contains('\x1b'));
        // The multi-line body collapses to one tidy line (no embedded \n).
        assert_eq!(
            line,
            "{\"file\":\"a.py\",\"line\":1,\"indent\":0,\"kind\":\"function\",\"text\":\"def f(a): return a\"}"
        );
    }
}
