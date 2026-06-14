//! Heuristic, dependency-free Lua signature extractor.
//!
//! Like [`super::python`] and [`super::braces`], this is a forward line scanner,
//! not a real parser. It first *masks* the source — blanking the contents of
//! comments (`--`, `--[[ ]]`) and string literals (`"…"`, `'…'`, long
//! `[[ ]]`/`[=[ ]=]`) while preserving line structure — so keywords hiding in
//! them never produce false positives. It then walks the masked lines, computes
//! nesting via an indentation stack, and recognizes declarations by shape:
//!
//!   * `function …`, `local function …`, `name = function(…)` -> Function
//!   * `Name = {…}` / `Name = setmetatable(…)` (type-like tables) -> Class
//!   * ALL-CAPS `NAME = …` module constants -> Constant
//!
//! Pathological inputs may be mis-handled; that is an accepted trade-off for the
//! zero-dependency goal. It never panics: all indexing is bounds-checked and the
//! scanner operates on chars so non-ASCII input is safe.

use super::Language;
use crate::signature::{Kind, Signature};

pub struct LuaLang;

impl Language for LuaLang {
    fn extract(&self, source: &str) -> Vec<Signature> {
        let masked = mask_lines(source);
        // Original lines (same per-line length as `masked`) so signature text can
        // keep literal content like a string table key `T["onClick"]`.
        let orig: Vec<&str> = source.lines().collect();
        let mut out = Vec::new();
        // Stack of indent widths of enclosing declarations, for nesting levels.
        let mut stack: Vec<usize> = Vec::new();
        let mut i = 0;

        while i < masked.len() {
            let line = &masked[i];
            let line_no = i + 1;
            let stripped = line.trim_start();
            if stripped.is_empty() {
                i += 1;
                continue;
            }
            let indent_width = leading_width(line);

            if is_function_start(stripped) {
                let (text, consumed) = gather_function(&masked, &orig, i);
                if !text.is_empty() {
                    let level = push_level(&mut stack, indent_width);
                    out.push(Signature { indent: level, kind: Kind::Function, text, line: line_no, full: None });
                }
                i += consumed.max(1);
                continue;
            }

            let orig_stripped = orig.get(i).map(|l| l.trim_start()).unwrap_or("");
            if let Some((kind, text, full)) = class_or_const(stripped, orig_stripped) {
                let level = push_level(&mut stack, indent_width);
                out.push(Signature { indent: level, kind, text, line: line_no, full });
            }

            i += 1;
        }

        out
    }
}

/// Mask every line of `source`: blank out the contents of comments and string
/// literals (replacing them with spaces) while preserving leading whitespace and
/// line structure. Long-bracket strings/comments (`[[ ]]`, `--[[ ]]`, `[=[ ]=]`)
/// carry their open state across lines. Operates on chars so it never splits a
/// multi-byte character.
fn mask_lines(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    // `Some(level)` while inside an open long bracket of the given `=` count.
    let mut long: Option<usize> = None;

    for raw in source.lines() {
        let chars: Vec<char> = raw.chars().collect();
        let len = chars.len();
        let mut masked = String::new();
        let mut j = 0;

        while j < len {
            // Inside an open long bracket: blank everything until the matching
            // closer `] =*{level} ]`.
            if let Some(level) = long {
                if chars[j] == ']' {
                    let mut k = j + 1;
                    let mut eq = 0;
                    while k < len && chars[k] == '=' {
                        eq += 1;
                        k += 1;
                    }
                    if eq == level && k < len && chars[k] == ']' {
                        long = None;
                        for _ in j..=k {
                            masked.push(' ');
                        }
                        j = k + 1;
                        continue;
                    }
                }
                masked.push(' ');
                j += 1;
                continue;
            }

            let c = chars[j];

            // Comment: `--` … or block comment `--[[ … ]]` / `--[=[ … ]=]`.
            if c == '-' && j + 1 < len && chars[j + 1] == '-' {
                let k = j + 2;
                if k < len && chars[k] == '[' {
                    let mut e = k + 1;
                    let mut eq = 0;
                    while e < len && chars[e] == '=' {
                        eq += 1;
                        e += 1;
                    }
                    if e < len && chars[e] == '[' {
                        long = Some(eq);
                        for _ in j..=e {
                            masked.push(' ');
                        }
                        j = e + 1;
                        continue;
                    }
                }
                // Plain line comment: blank the rest of the line.
                for _ in j..len {
                    masked.push(' ');
                }
                j = len;
                continue;
            }

            // Short string literal.
            if c == '"' || c == '\'' {
                let quote = c;
                masked.push(' ');
                j += 1;
                while j < len {
                    if chars[j] == '\\' {
                        masked.push(' ');
                        j += 1;
                        if j < len {
                            masked.push(' ');
                            j += 1;
                        }
                        continue;
                    }
                    let done = chars[j] == quote;
                    masked.push(' ');
                    j += 1;
                    if done {
                        break;
                    }
                }
                continue;
            }

            // Long string literal `[[ … ]]` / `[=[ … ]=]`.
            if c == '[' {
                let mut e = j + 1;
                let mut eq = 0;
                while e < len && chars[e] == '=' {
                    eq += 1;
                    e += 1;
                }
                if e < len && chars[e] == '[' {
                    long = Some(eq);
                    for _ in j..=e {
                        masked.push(' ');
                    }
                    j = e + 1;
                    continue;
                }
            }

            masked.push(c);
            j += 1;
        }

        out.push(masked);
    }

    out
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

/// Does this (masked, leading-trimmed) line begin a function declaration?
fn is_function_start(stripped: &str) -> bool {
    // `function …`
    if word_prefix(stripped, "function") {
        return true;
    }
    // `local function …`
    if let Some(rest) = strip_word(stripped, "local") {
        if word_prefix(rest.trim_start(), "function") {
            return true;
        }
    }
    // `lvalue = function(…)` / `local lvalue = function(…)`
    if let Some((_, rhs)) = split_assign(stripped) {
        if word_prefix(rhs.trim_start(), "function") {
            return true;
        }
    }
    false
}

/// Collect a function header starting at `start`, joining continuation lines
/// until the parameter parentheses balance. Returns the normalized one-line text
/// (body dropped) and the number of lines consumed.
fn gather_function(masked: &[String], orig: &[&str], start: usize) -> (String, usize) {
    let mut depth: i32 = 0;
    let mut seen_open = false;
    let mut pieces: Vec<String> = Vec::new();
    let mut consumed = 0;
    let mut done = false;

    for k in start..masked.len() {
        consumed += 1;
        // Structure (paren depth) comes from the masked line — so parens inside a
        // string/comment don't count — but the emitted text uses the original
        // chars at the same index so literal content (e.g. a string key) survives.
        let m: Vec<char> = masked[k].chars().collect();
        let o: Vec<char> = orig.get(k).map(|s| s.chars().collect()).unwrap_or_else(|| m.clone());
        let lead = m.iter().position(|c| !c.is_whitespace()).unwrap_or(m.len());

        let mut piece = String::new();
        let mut j = lead;
        while j < m.len() {
            let mc = m[j];
            let oc = if j < o.len() { o[j] } else { mc };
            match mc {
                '(' => {
                    depth += 1;
                    seen_open = true;
                    piece.push(oc);
                }
                ')' => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    piece.push(oc);
                    if seen_open && depth == 0 {
                        done = true;
                    }
                }
                _ => piece.push(oc),
            }
            j += 1;
            if done {
                break;
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

    let text = tidy(&collapse_ws(&pieces.join(" ")));
    (text, consumed)
}

/// If `stripped` is a type-like table (`Name = {…}` / `Name = setmetatable(…)`)
/// or an ALL-CAPS constant assignment, return its kind, normalized truncated text,
/// and optional full text (when a value was elided).
///
/// `stripped` is the **masked** line (string/comment contents blanked), used for
/// structural recognition. `orig_stripped` is the same line from the original
/// source, used to capture the real RHS for the `full` field.
fn class_or_const(stripped: &str, orig_stripped: &str) -> Option<(Kind, String, Option<String>)> {
    let (lhs, rhs) = split_assign(stripped)?;
    let lhs = lhs.trim();
    // `rhs` may be empty here when the value was a string literal that masking
    // blanked to spaces (e.g. `NAME = "app"`). That is still a constant, so do
    // not bail on an empty rhs — only an empty lhs is disqualifying.
    let rhs = rhs.trim_start();
    if lhs.is_empty() {
        return None;
    }

    // The bare name being assigned, with a leading `local ` stripped off.
    let name = strip_word(lhs, "local").map(|r| r.trim()).unwrap_or(lhs);

    // Type-like table: `Name = {…}` or `Name = setmetatable(…)` where the name
    // is capitalized (the common Lua class/module convention).
    if starts_upper(name) {
        if rhs.starts_with('{') {
            return Some((Kind::Class, format!("{lhs} = {{}}"), None));
        }
        if word_prefix(rhs, "setmetatable") {
            return Some((Kind::Class, format!("{lhs} = setmetatable(...)"), None));
        }
    }

    // Module constant: an ALL-CAPS identifier assigned a non-table value.
    if is_all_caps_ident(name) && !rhs.starts_with('{') {
        let prefix = if strip_word(lhs, "local").is_some() { "local " } else { "" };
        let text = format!("{prefix}{name} = \u{2026}");

        // Capture the real RHS from the original (unmasked) source line for `full`.
        let full = split_assign(orig_stripped).and_then(|(_, orig_rhs)| {
            let orig_rhs = orig_rhs.trim_start();
            if orig_rhs.is_empty() {
                None
            } else {
                Some(format!("{prefix}{name} = {}", collapse_ws(orig_rhs)))
            }
        });

        return Some((Kind::Constant, text, full));
    }

    None
}

/// Split `s` at the first top-level assignment `=`, rejecting relational
/// operators (`==`, `~=`, `<=`, `>=`). Returns `(lhs, rhs)` without the `=`.
fn split_assign(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
            let relational =
                matches!(prev, b'=' | b'~' | b'<' | b'>') || next == b'=';
            if !relational {
                // `=` is ASCII, so these split points are valid char boundaries.
                return Some((&s[..i], &s[i + 1..]));
            }
        }
        i += 1;
    }
    None
}

/// Does `s` start with the whole word `w` (not followed by an identifier char)?
fn word_prefix(s: &str, w: &str) -> bool {
    match s.strip_prefix(w) {
        Some(rest) => !matches!(rest.chars().next(), Some(c) if c.is_alphanumeric() || c == '_'),
        None => false,
    }
}

/// If `s` starts with the whole word `w`, return the remainder after it.
fn strip_word<'a>(s: &'a str, w: &str) -> Option<&'a str> {
    if word_prefix(s, w) {
        Some(&s[w.len()..])
    } else {
        None
    }
}

/// Does `s` begin with an ASCII uppercase letter?
fn starts_upper(s: &str) -> bool {
    matches!(s.chars().next(), Some(c) if c.is_ascii_uppercase())
}

/// Is `s` an ALL-CAPS identifier (`[A-Z][A-Z0-9_]*`)?
fn is_all_caps_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
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
        LuaLang.extract(src)
    }

    #[test]
    fn top_level_functions() {
        let src = "function foo(a, b)\n  return a\nend\n\nlocal function bar()\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].kind, Kind::Function);
        assert_eq!(s[0].text, "function foo(a, b)");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[0].line, 1);
        assert_eq!(s[1].text, "local function bar()");
        assert_eq!(s[1].line, 5);
    }

    #[test]
    fn method_and_assigned_function() {
        let src = "function M.foo(a)\nend\nM.bar = function(x, y)\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].text, "function M.foo(a)");
        assert_eq!(s[1].text, "M.bar = function(x, y)");
    }

    #[test]
    fn string_key_function_preserves_key() {
        // The string key inside `T["k"]` is masked for structure but must survive
        // in the emitted signature text.
        let src = "T[\"onClick\"] = function(event)\nend\nT['k'] = function(x)\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].text, "T[\"onClick\"] = function(event)");
        assert_eq!(s[1].text, "T['k'] = function(x)");
    }

    #[test]
    fn nested_function_has_indent() {
        let src = "function outer()\n  local function inner(x)\n    return x\n  end\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].text, "function outer()");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[1].text, "local function inner(x)");
        assert_eq!(s[1].indent, 1);
    }

    #[test]
    fn multiline_signature_joined() {
        let src = "function f(\n  a,\n  b,\n  c\n)\n  return a\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "function f(a, b, c)");
    }

    #[test]
    fn class_table_detected() {
        let src = "Account = {}\nlocal Animal = {\n  legs = 4,\n}\n";
        let s = sigs(src);
        let classes: Vec<String> = s
            .iter()
            .filter(|x| x.kind == Kind::Class)
            .map(|x| x.text.clone())
            .collect();
        assert_eq!(classes, vec!["Account = {}".to_string(), "local Animal = {}".to_string()]);
    }

    #[test]
    fn constants_detected_and_rejected() {
        let src = "MAX_SIZE = 10\nPI = 3.14\nlocal x = 1\nif X == Y then end\nconfig = compute()\n";
        let texts: Vec<String> = sigs(src)
            .iter()
            .filter(|x| x.kind == Kind::Constant)
            .map(|x| x.text.clone())
            .collect();
        assert_eq!(texts, vec!["MAX_SIZE = …".to_string(), "PI = …".to_string()]);
    }

    #[test]
    fn decl_in_comment_ignored() {
        let src = "-- function fake()\n--[[ function nope()\nstill in comment ]]\nfunction real()\nend\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "function real()");
        assert_eq!(s[0].line, 4);
    }

    #[test]
    fn decl_in_string_ignored() {
        let src = "local s = \"function hidden()\"\nlocal t = [[ function alsohidden() ]]\n";
        let s = sigs(src);
        assert!(s.is_empty(), "got: {s:?}");
    }

    #[test]
    fn non_ascii_does_not_panic() {
        let src = "-- naïve café αβγ\nfunction grüße(café)\nend\nNAMÉ = 1\n";
        let s = sigs(src);
        assert_eq!(s[0].text, "function grüße(café)");
    }

    #[test]
    fn string_valued_constants_emitted() {
        // The string value masks to spaces; an ALL-CAPS assignment is still a
        // constant (previously dropped because the masked rhs looked empty).
        let src = "NAME = \"app\"\nVERSION = \"1.0\"\nMAX = 10\nlower = \"x\"\n";
        let consts: Vec<String> = sigs(src)
            .into_iter()
            .filter(|x| x.kind == Kind::Constant)
            .map(|x| x.text)
            .collect();
        assert_eq!(
            consts,
            vec![
                "NAME = …".to_string(),
                "VERSION = …".to_string(),
                "MAX = …".to_string(),
            ]
        );
    }

    #[test]
    fn constants_full_field_captures_real_value() {
        // `text` must keep "…" but `full` must contain the real RHS.
        let src = "MAX_SIZE = 1 << 20\nPI = 3.14\nVERSION = \"1.0.0\"\nlocal LIMIT = 100\n";
        let consts: Vec<Signature> = sigs(src)
            .into_iter()
            .filter(|x| x.kind == Kind::Constant)
            .collect();
        // Truncated text still uses elision.
        assert_eq!(consts[0].text, "MAX_SIZE = \u{2026}");
        assert_eq!(consts[1].text, "PI = \u{2026}");
        assert_eq!(consts[2].text, "VERSION = \u{2026}");
        assert_eq!(consts[3].text, "local LIMIT = \u{2026}");
        // Full field contains the real value.
        assert_eq!(consts[0].full, Some("MAX_SIZE = 1 << 20".to_string()));
        assert_eq!(consts[1].full, Some("PI = 3.14".to_string()));
        // String literal value preserved in full.
        assert_eq!(consts[2].full, Some("VERSION = \"1.0.0\"".to_string()));
        // local prefix is kept in full.
        assert_eq!(consts[3].full, Some("local LIMIT = 100".to_string()));
    }
}
