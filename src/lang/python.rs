//! Heuristic, dependency-free Python signature extractor.
//!
//! This is a forward line scanner, not a real parser. Source is first masked —
//! comments and string literals (single/double/triple, with escapes and raw
//! prefixes) are blanked so their contents never look like declarations — and
//! the structural scan then runs over the masked text while declaration text is
//! emitted from the original source (so string defaults survive verbatim).
//! Indentation determines nesting; constants inside a function body are dropped.
//! Pathological inputs may be mis-handled; that is an accepted trade-off for the
//! zero-dependency goal. The scanner never panics and operates on whole `char`s.

use super::Language;
use crate::signature::{Kind, Signature};

/// Per-character classification produced by masking.
const CODE: u8 = 0;
const STR: u8 = 1;
const COMMENT: u8 = 2;

pub struct PythonLang;

impl Language for PythonLang {
    fn extract(&self, source: &str) -> Vec<Signature> {
        let olines: Vec<&str> = source.lines().collect();
        let (mlines, blines) = mask_all(&olines);

        let mut out = Vec::new();
        // Stack of (indent width, is_function_body) for enclosing declarations.
        let mut stack: Vec<(usize, bool)> = Vec::new();
        let mut i = 0;

        while i < olines.len() {
            let mstripped = mlines[i].trim_start();
            if mstripped.is_empty() {
                i += 1;
                continue;
            }

            let indent_width = leading_width(olines[i]);
            let line_no = i + 1;

            if let Some(kind) = def_kind(mstripped) {
                let (text, consumed) = gather_def(&mlines, &olines, &blines, i);
                let level = push_level(&mut stack, indent_width, kind == Kind::Function);
                out.push(Signature { indent: level, kind, text, line: line_no });
                i += consumed;
                continue;
            }

            if let Some(text) = constant_sig(mstripped, olines[i].trim_start()) {
                // A constant is only a module/class attribute, never a local: pop
                // exited scopes and suppress it if it sits inside a function body.
                pop_to(&mut stack, indent_width);
                let suppress = stack.last().map_or(false, |&(_, is_fn)| is_fn);
                if !suppress {
                    let level = stack.len();
                    out.push(Signature {
                        indent: level,
                        kind: Kind::Constant,
                        text,
                        line: line_no,
                    });
                }
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

/// Pop stack entries at or deeper than `width` (their scopes have been exited).
fn pop_to(stack: &mut Vec<(usize, bool)>, width: usize) {
    while let Some(&(top, _)) = stack.last() {
        if top >= width {
            stack.pop();
        } else {
            break;
        }
    }
}

/// Pop exited scopes, return the nesting level for a declaration at `width`, and
/// push it (recording whether it introduces a function body).
fn push_level(stack: &mut Vec<(usize, bool)>, width: usize, is_func: bool) -> usize {
    pop_to(stack, width);
    let level = stack.len();
    stack.push((width, is_func));
    level
}

/// Classify a masked, left-trimmed line as the start of a `def`/`class`.
/// Tolerates extra whitespace (`async   def`).
fn def_kind(stripped: &str) -> Option<Kind> {
    let mut it = stripped.split_whitespace();
    match it.next() {
        Some("def") => Some(Kind::Function),
        Some("class") => Some(Kind::Class),
        Some("async") if it.next() == Some("def") => Some(Kind::Function),
        _ => None,
    }
}

/// Mask every source line, carrying triple-quoted string state across lines.
/// Returns the masked lines and a per-char classification (`CODE`/`STR`/
/// `COMMENT`) aligned 1:1 with each original line's chars.
fn mask_all(olines: &[&str]) -> (Vec<String>, Vec<Vec<u8>>) {
    let mut triple: Option<[char; 3]> = None;
    let mut mlines = Vec::with_capacity(olines.len());
    let mut blines = Vec::with_capacity(olines.len());
    for line in olines {
        let (m, b) = mask_line(line, &mut triple);
        mlines.push(m);
        blines.push(b);
    }
    (mlines, blines)
}

/// Mask one line. `triple` carries an open triple-quote delimiter across lines.
fn mask_line(line: &str, triple: &mut Option<[char; 3]>) -> (String, Vec<u8>) {
    let cs: Vec<char> = line.chars().collect();
    let n = cs.len();
    let mut m = String::with_capacity(n);
    let mut b = Vec::with_capacity(n);
    let mut i = 0;

    while i < n {
        if let Some(delim) = *triple {
            if i + 3 <= n && cs[i] == delim[0] && cs[i + 1] == delim[1] && cs[i + 2] == delim[2] {
                *triple = None;
                for _ in 0..3 {
                    m.push(' ');
                    b.push(STR);
                }
                i += 3;
            } else {
                m.push(' ');
                b.push(STR);
                i += 1;
            }
            continue;
        }

        let c = cs[i];
        if c == '#' {
            while i < n {
                m.push(' ');
                b.push(COMMENT);
                i += 1;
            }
            break;
        }

        // Triple-quoted string opener (checked before single quotes).
        if i + 3 <= n
            && ((cs[i] == '"' && cs[i + 1] == '"' && cs[i + 2] == '"')
                || (cs[i] == '\'' && cs[i + 1] == '\'' && cs[i + 2] == '\''))
        {
            *triple = Some([cs[i], cs[i], cs[i]]);
            for _ in 0..3 {
                m.push(' ');
                b.push(STR);
            }
            i += 3;
            continue;
        }

        // Single/double-quoted single-line string.
        if c == '"' || c == '\'' {
            // A raw string has an `r`/`R` in its prefix. Single-char prefixes
            // (`r"`) put it directly before the quote; two-char prefixes like
            // `rb"`/`rf"`/`br"`/`fr"` put a `b`/`f` between the `r` and the
            // quote, so check the char before that too.
            let raw = (i > 0 && matches!(cs[i - 1], 'r' | 'R'))
                || (i > 1
                    && matches!(cs[i - 1], 'b' | 'B' | 'f' | 'F')
                    && matches!(cs[i - 2], 'r' | 'R'));
            m.push(' ');
            b.push(STR);
            i += 1;
            while i < n {
                let cc = cs[i];
                if cc == '\\' && !raw && i + 1 < n {
                    m.push(' ');
                    b.push(STR);
                    m.push(' ');
                    b.push(STR);
                    i += 2;
                    continue;
                }
                m.push(' ');
                b.push(STR);
                i += 1;
                if cc == c {
                    break;
                }
            }
            continue;
        }

        m.push(c);
        b.push(CODE);
        i += 1;
    }

    (m, b)
}

/// Collect a `def`/`class` declaration starting at `start`, joining continuation
/// lines until the parameter brackets balance and the signature-terminating `:`
/// is found at top level. Structure is read from the masked lines; the emitted
/// text is taken from the original source (string contents preserved). Returns
/// the normalized text (trailing `:`, body dropped) and lines consumed.
fn gather_def(
    mlines: &[String],
    olines: &[&str],
    blines: &[Vec<u8>],
    start: usize,
) -> (String, usize) {
    let mut depth: i32 = 0;
    let mut seq: Vec<(char, bool)> = Vec::new();
    let mut consumed = 0;
    let mut done = false;

    for k in start..mlines.len() {
        consumed += 1;
        let mchars: Vec<char> = mlines[k].chars().collect();
        let ochars: Vec<char> = olines[k].chars().collect();
        let flags = &blines[k];

        // Skip leading whitespace using the ORIGINAL chars, not the masked
        // ones: a continuation line whose content is entirely inside a string
        // is masked to spaces, so skipping on `mchars` would discard the whole
        // line (losing string defaults). Skipping on `ochars` stops at the
        // first non-whitespace source char — be it code or string content.
        let mut j = 0;
        while j < ochars.len() && ochars[j].is_whitespace() {
            j += 1;
        }

        while j < mchars.len() {
            let mc = mchars[j];
            match mc {
                '(' | '[' | '{' => {
                    depth += 1;
                    seq.push((ochars[j], false));
                    j += 1;
                }
                ')' | ']' | '}' => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    seq.push((ochars[j], false));
                    j += 1;
                }
                ':' if depth <= 0 => {
                    done = true;
                    break;
                }
                _ => {
                    match flags.get(j).copied().unwrap_or(CODE) {
                        COMMENT => {} // drop comment text entirely
                        STR => seq.push((ochars[j], true)),
                        _ => seq.push((ochars[j], false)),
                    }
                    j += 1;
                }
            }
        }

        // Drop trailing code whitespace and an explicit `\` line continuation.
        while seq.last().map_or(false, |&(c, s)| !s && c.is_whitespace()) {
            seq.pop();
        }
        if seq.last() == Some(&('\\', false)) {
            seq.pop();
        }

        if done || consumed > 2000 {
            break;
        }
        // Separate this line from the next joined one.
        seq.push((' ', false));
    }

    let mut text = normalize(&seq);
    text.push(':');
    (text, consumed)
}

/// Collapse code whitespace and tidy bracket/comma spacing, leaving the contents
/// of string literals (`is_str == true`) untouched.
fn normalize(seq: &[(char, bool)]) -> String {
    // 1. Collapse runs of non-string whitespace to a single space.
    let mut v: Vec<(char, bool)> = Vec::with_capacity(seq.len());
    let mut prev_ws = false;
    for &(c, s) in seq {
        if !s && c.is_whitespace() {
            if !prev_ws {
                v.push((' ', false));
                prev_ws = true;
            }
        } else {
            v.push((c, s));
            prev_ws = false;
        }
    }
    while v.first().map_or(false, |&(c, s)| !s && c == ' ') {
        v.remove(0);
    }
    while v.last().map_or(false, |&(c, s)| !s && c == ' ') {
        v.pop();
    }

    // 2. Tidy: drop spaces hugging brackets/commas/colons and trailing commas.
    let n = v.len();
    let mut out = String::new();
    let mut i = 0;
    while i < n {
        let (c, s) = v[i];
        if !s && c == ' ' {
            let prev_opener = out.chars().last().map_or(false, |p| matches!(p, '(' | '[' | '{'));
            let mut k = i + 1;
            while k < n && !v[k].1 && v[k].0 == ' ' {
                k += 1;
            }
            let next_close = k < n && !v[k].1 && matches!(v[k].0, ')' | ']' | '}' | ',' | ':');
            if prev_opener || next_close {
                i += 1;
                continue;
            }
        }
        if !s && c == ',' {
            let mut k = i + 1;
            while k < n && !v[k].1 && v[k].0 == ' ' {
                k += 1;
            }
            if k < n && !v[k].1 && matches!(v[k].0, ')' | ']' | '}') {
                i += 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// If `mstripped` is a module/class-level constant assignment, return its
/// normalized signature (`NAME[: Type] = …`). `ostripped` is the original text
/// (same char layout), used to preserve the annotation verbatim.
///
/// A constant is an ALL-CAPS identifier (`[A-Z][A-Z0-9_]*`), optionally
/// `: Type`, followed by a single top-level `=`. Comparisons / augmented forms
/// are rejected.
fn constant_sig(mstripped: &str, ostripped: &str) -> Option<String> {
    let mc: Vec<char> = mstripped.chars().collect();
    let oc: Vec<char> = ostripped.chars().collect();
    if oc.is_empty() || !oc[0].is_ascii_uppercase() {
        return None;
    }

    let mut i = 0;
    while i < oc.len() {
        let c = oc[i];
        if c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' {
            i += 1;
        } else {
            break;
        }
    }
    let name: String = oc[..i].iter().collect();

    let mut j = i;
    while j < mc.len() && mc[j].is_whitespace() {
        j += 1;
    }

    let (annotation, eq) = if j < mc.len() && mc[j] == ':' {
        let e = find_top_eq(&mc, j + 1)?;
        let ann: String = oc[j + 1..e].iter().collect();
        (ann.trim().to_string(), e)
    } else if j < mc.len() && mc[j] == '=' {
        (String::new(), j)
    } else {
        return None;
    };

    // Reject `==` comparisons.
    if eq + 1 < mc.len() && mc[eq + 1] == '=' {
        return None;
    }

    let mut text = name;
    if !annotation.is_empty() {
        text.push_str(": ");
        text.push_str(&annotation);
    }
    text.push_str(" = …");
    Some(text)
}

/// Find the first top-level (bracket-depth-zero) plain `=` at or after `from`,
/// skipping `==` and comparison/augmented operators.
fn find_top_eq(mc: &[char], from: usize) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut k = from;
    while k < mc.len() {
        match mc[k] {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '=' if depth <= 0 => {
                let next_eq = k + 1 < mc.len() && mc[k + 1] == '=';
                let prev = if k > 0 { mc[k - 1] } else { ' ' };
                let augmented = matches!(
                    prev,
                    '<' | '>' | '!' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~' | ':' | '='
                );
                if !next_eq && !augmented {
                    return Some(k);
                }
            }
            _ => {}
        }
        k += 1;
    }
    None
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

    #[test]
    fn async_extra_spaces() {
        let s = sigs("async  def bar():\n    pass\n");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "async def bar():");
    }

    #[test]
    fn string_default_preserved() {
        let s = sigs("def f(x=\" ,hello\"):\n    pass\n");
        assert_eq!(s[0].text, "def f(x=\" ,hello\"):");
    }

    #[test]
    fn bracket_and_colon_in_string_not_terminator() {
        let a = sigs("def f(x=\"(\"):\n    pass\n");
        assert_eq!(a[0].text, "def f(x=\"(\"):");
        let b = sigs("def f() -> \"A: B\":\n    pass\n");
        assert_eq!(b[0].text, "def f() -> \"A: B\":");
    }

    #[test]
    fn comment_in_params_stripped() {
        let src = "def f(\n    a,  # one\n    b,\n) -> None:\n    pass\n";
        let s = sigs(src);
        assert_eq!(s[0].text, "def f(a, b) -> None:");
    }

    #[test]
    fn backslash_continuation_joined() {
        let s = sigs("def f(a, \\\n      b):\n    pass\n");
        assert_eq!(s[0].text, "def f(a, b):");
    }

    #[test]
    fn annotation_with_equals_inside_brackets() {
        let s = sigs("MY_CONST: Annotated[int, Field(default=0)] = 0\n");
        assert_eq!(s[0].text, "MY_CONST: Annotated[int, Field(default=0)] = …");
    }

    #[test]
    fn local_const_suppressed() {
        let s = sigs("def f():\n    LOCAL = 42\n    return LOCAL\n");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "def f():");
    }

    #[test]
    fn multiline_triple_quoted_default_preserved() {
        let s = sigs("def f(\n    x=\"\"\"\n    multi\n    \"\"\",\n) -> str:\n    return x\n");
        assert_eq!(s[0].text, "def f(x=\"\"\" multi \"\"\") -> str:");
    }

    #[test]
    fn multiline_string_collection_default_preserved() {
        let s = sigs("def f(\n    items=[\n        \"a\",\n        \"b\",\n    ],\n) -> list:\n    pass\n");
        assert_eq!(s[0].text, "def f(items=[\"a\", \"b\"]) -> list:");
    }

    #[test]
    fn raw_two_char_prefix_string_closes() {
        let s = sigs("def f(x=rb\"ends_with_backslash\\\"):\n    pass\n\ndef g() -> int:\n    return 1\n");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].text, "def f(x=rb\"ends_with_backslash\\\"):");
        assert_eq!(s[1].text, "def g() -> int:");
    }

    #[test]
    fn triple_marker_inside_quotes_not_triple() {
        let s = sigs("def a():\n    x = \"'''\"\n\ndef b():\n    pass\n");
        let texts: Vec<String> = s.iter().map(|x| x.text.clone()).collect();
        assert_eq!(texts, vec!["def a():".to_string(), "def b():".to_string()]);
    }
}
