//! Heuristic, dependency-free Python signature extractor.
//!
//! This is a forward line scanner, not a real parser. It tracks triple-quoted
//! string state (so `def`/`class` text inside docstrings is ignored) and uses
//! indentation to compute nesting levels. Pathological inputs may be
//! mis-handled; that is an accepted trade-off for the zero-dependency goal.

use super::Language;
use crate::signature::{Kind, Signature};

pub struct PythonLang;

impl Language for PythonLang {
    fn extract(&self, source: &str) -> Vec<Signature> {
        let lines: Vec<&str> = source.lines().collect();
        let mut out = Vec::new();
        // Current open triple-quote delimiter, if we are inside a string.
        let mut triple: Option<&'static str> = None;
        // Stack of indent widths of enclosing declarations, for nesting levels.
        let mut stack: Vec<usize> = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let raw = lines[i];
            let line_no = i + 1;

            // Advance string state for this line; skip lines that begin inside
            // a triple-quoted string.
            if update_triple(raw, &mut triple) {
                i += 1;
                continue;
            }

            let indent_width = leading_width(raw);
            let stripped = raw.trim_start();

            let kind = if stripped.starts_with("def ") || stripped.starts_with("async def ") {
                Some(Kind::Function)
            } else if stripped.starts_with("class ") {
                Some(Kind::Class)
            } else {
                None
            };

            if let Some(kind) = kind {
                let (text, consumed) = gather_def(&lines, i);
                let level = push_level(&mut stack, indent_width);
                out.push(Signature { indent: level, kind, text, line: line_no });
                // Keep string state consistent across the lines we swallowed.
                for j in (i + 1)..(i + consumed) {
                    update_triple(lines[j], &mut triple);
                }
                i += consumed;
                continue;
            }

            if let Some(text) = constant_sig(stripped) {
                let level = push_level(&mut stack, indent_width);
                out.push(Signature { indent: level, kind: Kind::Constant, text, line: line_no });
            }

            i += 1;
        }

        out
    }
}

/// Width of the leading whitespace, counting a tab as four columns.
fn leading_width(line: &str) -> usize {
    let mut w = 0;
    for c in line.chars() {
        match c {
            ' ' => w += 1,
            '\t' => w += 4,
            _ => break,
        }
    }
    w
}

/// Given the indent width of a new declaration, pop deeper/equal entries off the
/// stack, return the resulting nesting level, and record this declaration.
fn push_level(stack: &mut Vec<usize>, width: usize) -> usize {
    while let Some(&top) = stack.last() {
        if top >= width {
            stack.pop();
        } else {
            break;
        }
    }
    let level = stack.len();
    stack.push(width);
    level
}

/// Update triple-quote string state for one line. Returns `true` if the line
/// *began* inside a triple-quoted string (and should therefore be ignored).
/// Operates on bytes to stay panic-free on non-ASCII input.
fn update_triple(line: &str, state: &mut Option<&'static str>) -> bool {
    let began_inside = state.is_some();
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match *state {
            Some(delim) => {
                let d = delim.as_bytes();
                if i + 3 <= b.len() && &b[i..i + 3] == d {
                    *state = None;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            None => {
                if i + 3 <= b.len() && &b[i..i + 3] == b"\"\"\"" {
                    *state = Some("\"\"\"");
                    i += 3;
                } else if i + 3 <= b.len() && &b[i..i + 3] == b"'''" {
                    *state = Some("'''");
                    i += 3;
                } else if b[i] == b'#' {
                    // Rest of the line is a comment; stop scanning.
                    break;
                } else {
                    i += 1;
                }
            }
        }
    }
    began_inside
}

/// Collect a `def`/`class` declaration starting at `start`, joining any
/// continuation lines until the parameter parentheses balance and the
/// signature-terminating colon is found. Returns the normalized text (with a
/// trailing `:`, body dropped) and the number of lines consumed.
fn gather_def(lines: &[&str], start: usize) -> (String, usize) {
    let mut depth: i32 = 0;
    let mut pieces: Vec<String> = Vec::new();
    let mut consumed = 0;
    let mut done = false;

    for k in start..lines.len() {
        consumed += 1;
        let src = if k == start {
            lines[k].trim_start()
        } else {
            lines[k].trim()
        };

        let mut piece = String::new();
        for c in src.chars() {
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
                ':' if depth <= 0 => {
                    done = true;
                    break;
                }
                '#' if depth <= 0 => break,
                _ => piece.push(c),
            }
        }

        let piece = piece.trim_end().to_string();
        if !piece.is_empty() {
            pieces.push(piece);
        }
        if done || k - start > 100 {
            break;
        }
    }

    let mut text = tidy(&collapse_ws(&pieces.join(" ")));
    text.push(':');
    (text, consumed)
}

/// If `stripped` is a module/class-level constant assignment, return its
/// normalized signature (name, optional type annotation, elided value).
///
/// A constant is an ALL-CAPS identifier (`[A-Z][A-Z0-9_]*`), optionally
/// `: Type`, followed by a single `=`. Augmented assignments and comparisons
/// (`==`, `+=`, …) are rejected.
fn constant_sig(stripped: &str) -> Option<String> {
    let bytes = stripped.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_uppercase() {
        return None;
    }

    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_' {
            i += 1;
        } else {
            break;
        }
    }
    let name = &stripped[..i];
    let rest = stripped[i..].trim_start();

    // Optional type annotation: `: Type`.
    let (annotation, after) = if let Some(r) = rest.strip_prefix(':') {
        let eq = r.find('=')?;
        (Some(r[..eq].trim().to_string()), &r[eq..])
    } else {
        (None, rest)
    };

    let after = after.trim_start();
    // Must be a plain assignment, not `==` (comparison). Augmented forms like
    // `+=` never reach here because the operator char breaks identifier parsing.
    if !after.starts_with('=') || after.starts_with("==") {
        return None;
    }

    let mut text = String::from(name);
    if let Some(a) = annotation {
        if !a.is_empty() {
            text.push_str(": ");
            text.push_str(&a);
        }
    }
    text.push_str(" = …");
    Some(text)
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

/// Tidy up spacing introduced when joining a multi-line signature: remove spaces
/// hugging brackets/commas/colons and drop trailing commas before a closer.
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
        (" :", ":"),
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
        PythonLang.extract(src)
    }

    #[test]
    fn top_level_functions() {
        let s = sigs("def foo(a, b):\n    return a\n\nasync def bar():\n    pass\n");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].kind, Kind::Function);
        assert_eq!(s[0].text, "def foo(a, b):");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[0].line, 1);
        assert_eq!(s[1].text, "async def bar():");
        assert_eq!(s[1].line, 4);
    }

    #[test]
    fn class_with_methods_nested() {
        let src = "class Foo(Base):\n    def __init__(self):\n        pass\n    def run(self, x):\n        return x\n";
        let s = sigs(src);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[0].text, "class Foo(Base):");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[1].text, "def __init__(self):");
        assert_eq!(s[1].indent, 1);
        assert_eq!(s[2].text, "def run(self, x):");
        assert_eq!(s[2].indent, 1);
    }

    #[test]
    fn multiline_signature_joined() {
        let src = "def f(\n    a,\n    b,\n    c,\n):\n    pass\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "def f(a, b, c):");
    }

    #[test]
    fn return_annotation_kept() {
        let s = sigs("def g(x: int) -> Dict[str, int]:\n    pass\n");
        assert_eq!(s[0].text, "def g(x: int) -> Dict[str, int]:");
    }

    #[test]
    fn constants_detected_and_rejected() {
        let src = "MAX_SIZE = 10\nMODE: str = \"x\"\nlowercase = 1\nif X == Y:\n    pass\n";
        let texts: Vec<String> = sigs(src)
            .iter()
            .filter(|x| x.kind == Kind::Constant)
            .map(|x| x.text.clone())
            .collect();
        assert_eq!(texts, vec!["MAX_SIZE = …".to_string(), "MODE: str = …".to_string()]);
    }

    #[test]
    fn def_in_docstring_ignored() {
        let src = "def real():\n    \"\"\"\n    def fake():\n    \"\"\"\n    pass\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "def real():");
    }
}
