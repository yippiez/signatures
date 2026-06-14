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
        Format::Jsonl => {
            // Never colorized. Compact, hand-rolled JSON object.
            let kind = match s.kind {
                Kind::Function => "function",
                Kind::Class => "class",
                Kind::Constant => "constant",
            };
            let mut o = String::new();
            o.push_str("{\"file\":");
            json_string(path, &mut o);
            o.push_str(",\"line\":");
            o.push_str(&s.line.to_string());
            o.push_str(",\"indent\":");
            o.push_str(&s.indent.to_string());
            o.push_str(",\"kind\":\"");
            o.push_str(kind);
            o.push_str("\",\"text\":");
            json_string(text, &mut o);
            o.push('}');
            o
        }
    }
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
        Signature { indent: 0, kind, text: text.to_string(), line: 1, full: None }
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
        let colors = Colors { enabled: true };
        let s = sig(Kind::Constant, "MAX = …");
        let line = render_one("a.py", &s, &colors, Format::Jsonl, OutputMode::Truncated);
        assert!(!line.contains('\x1b'), "jsonl must never be colorized: {line:?}");
        assert_eq!(
            line,
            "{\"file\":\"a.py\",\"line\":1,\"indent\":0,\"kind\":\"constant\",\"text\":\"MAX = …\"}"
        );
    }

    #[test]
    fn full_falls_back_to_text_when_none() {
        let colors = Colors { enabled: false };
        let s = sig(Kind::Constant, "MAX = …");
        let trunc = render_one("a.py", &s, &colors, Format::Plain, OutputMode::Truncated);
        let full = render_one("a.py", &s, &colors, Format::Plain, OutputMode::Full);
        assert_eq!(trunc, full);
    }

    #[test]
    fn full_expands_when_some() {
        let colors = Colors { enabled: false };
        let mut s = sig(Kind::Constant, "MAX = …");
        s.full = Some("MAX = 1 << 20".to_string());
        let full = render_one("a.py", &s, &colors, Format::Plain, OutputMode::Full);
        assert_eq!(full, "MAX = 1 << 20");
        let trunc = render_one("a.py", &s, &colors, Format::Plain, OutputMode::Truncated);
        assert_eq!(trunc, "MAX = …");
    }
}
