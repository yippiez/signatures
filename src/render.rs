//! Turn extracted signatures into colorized, indented output text.

use crate::color::Colors;
use crate::signature::{Kind, Signature};

/// Append the rendered block for one file (optional header + its signatures) to
/// `out`.
pub fn render(path: &str, sigs: &[Signature], colors: &Colors, show_header: bool, out: &mut String) {
    if show_header {
        out.push_str(&colors.header(path));
        out.push('\n');
    }
    for s in sigs {
        for _ in 0..s.indent {
            out.push_str("  ");
        }
        out.push_str(&colorize(s.kind, &s.text, colors));
        out.push('\n');
    }
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
        Signature { indent: 0, kind, text: text.to_string(), line: 1 }
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
        render("a.py", &[sig(Kind::Function, "def run(self):")], &colors, false, &mut out);
        assert!(out.contains("\x1b[1;35mdef\x1b[0m"));
        assert!(out.contains("\x1b[1;36mrun\x1b[0m"));
    }

    #[test]
    fn nesting_indents() {
        let colors = Colors { enabled: false };
        let mut out = String::new();
        let mut s = sig(Kind::Function, "def m(self):");
        s.indent = 1;
        render("a.py", &[s], &colors, false, &mut out);
        assert!(out.starts_with("  def m(self):"));
    }
}
