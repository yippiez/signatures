//! Heuristic, dependency-free Bash (shell) signature extractor.
//!
//! This is a forward line scanner, not a real parser. It scrubs comments,
//! quoted strings and `$(...)`/`${...}` expansions, then tracks brace nesting to
//! compute the level of each `name() { … }` / `function name { … }` definition.
//! Module-level ALL-CAPS assignments (optionally `readonly`/`declare -r`/…) are
//! reported as constants. Bash has no class/type construct, so no `Class`
//! signatures are produced. Pathological inputs (heredocs, backtick command
//! substitution spanning lines) may be mis-handled; that is an accepted
//! trade-off for the zero-dependency goal. The scanner never panics and always
//! operates on whole `char`s.

use super::Language;
use crate::signature::{Kind, Signature};

pub struct BashLang;

impl Language for BashLang {
    fn extract(&self, source: &str) -> Vec<Signature> {
        // Pre-scrub every line, carrying quote state across line boundaries so
        // multi-line strings hide any `def`-like text inside them.
        let mut quote: Option<char> = None;
        let cleaned: Vec<String> =
            source.lines().map(|l| scrub_line(l, &mut quote)).collect();

        let mut out = Vec::new();
        // Brace nesting depth (clamped at >= 0).
        let mut depth: i32 = 0;
        // Recorded depth of each currently-open enclosing function definition.
        let mut stack: Vec<i32> = Vec::new();
        let mut i = 0;

        while i < cleaned.len() {
            // Drop any enclosing functions whose body has already closed.
            while let Some(&d) = stack.last() {
                if d >= depth {
                    stack.pop();
                } else {
                    break;
                }
            }

            let line = &cleaned[i];
            let trimmed = line.trim_start();
            let line_no = i + 1;

            if trimmed.is_empty() {
                depth = bump(depth, brace_delta(line));
                i += 1;
                continue;
            }

            if is_func_start(trimmed) {
                let level = stack.len();
                let (text, consumed) = gather_func(&cleaned, i);
                out.push(Signature { indent: level, kind: Kind::Function, text, line: line_no });
                stack.push(depth);
                for k in i..(i + consumed) {
                    depth = bump(depth, brace_delta(&cleaned[k]));
                }
                i += consumed;
                continue;
            }

            // Constants are only meaningful at module scope (not inside a
            // function body).
            if stack.is_empty() && depth == 0 {
                if let Some(text) = constant_sig(trimmed) {
                    out.push(Signature { indent: 0, kind: Kind::Constant, text, line: line_no });
                }
            }

            depth = bump(depth, brace_delta(line));
            i += 1;
        }

        out
    }
}

/// Add a brace delta to the running depth, clamping at zero so unbalanced input
/// never drives the nesting stack negative.
fn bump(depth: i32, delta: i32) -> i32 {
    let d = depth + delta;
    if d < 0 {
        0
    } else {
        d
    }
}

/// Net `{` minus `}` count on an already-scrubbed line.
fn brace_delta(line: &str) -> i32 {
    let mut d = 0;
    for c in line.chars() {
        match c {
            '{' => d += 1,
            '}' => d -= 1,
            _ => {}
        }
    }
    d
}

/// Remove the parts of a line that must not be scanned for declarations or
/// braces: `#` comments, single/double-quoted strings, `${…}` parameter
/// expansions and `$(…)` command/arithmetic substitutions. Removed regions are
/// replaced with spaces. `quote` carries an open single/double quote across
/// lines. Operates on whole `char`s so non-ASCII input is safe.
fn scrub_line(line: &str, quote: &mut Option<char>) -> String {
    let cs: Vec<char> = line.chars().collect();
    let n = cs.len();
    let mut out = String::with_capacity(n);
    let mut i = 0;

    while i < n {
        let c = cs[i];

        if let Some(q) = *quote {
            if q == '\'' {
                if c == '\'' {
                    *quote = None;
                }
                out.push(' ');
                i += 1;
            } else {
                // Inside a double-quoted string.
                if c == '\\' && i + 1 < n {
                    out.push(' ');
                    out.push(' ');
                    i += 2;
                } else if c == '"' {
                    *quote = None;
                    out.push(' ');
                    i += 1;
                } else {
                    out.push(' ');
                    i += 1;
                }
            }
            continue;
        }

        match c {
            '#' if i == 0 || cs[i - 1].is_whitespace() => break,
            '\'' => {
                *quote = Some('\'');
                out.push(' ');
                i += 1;
            }
            '"' => {
                *quote = Some('"');
                out.push(' ');
                i += 1;
            }
            '\\' => {
                out.push(' ');
                if i + 1 < n {
                    out.push(' ');
                    i += 2;
                } else {
                    i += 1;
                }
            }
            '$' if i + 1 < n && cs[i + 1] == '{' => {
                out.push(' ');
                out.push(' ');
                i += 2;
                let mut nesting = 1;
                while i < n && nesting > 0 {
                    match cs[i] {
                        '{' => nesting += 1,
                        '}' => nesting -= 1,
                        _ => {}
                    }
                    out.push(' ');
                    i += 1;
                }
            }
            '$' if i + 1 < n && cs[i + 1] == '(' => {
                out.push(' ');
                out.push(' ');
                i += 2;
                let mut nesting = 1;
                while i < n && nesting > 0 {
                    match cs[i] {
                        '(' => nesting += 1,
                        ')' => nesting -= 1,
                        _ => {}
                    }
                    out.push(' ');
                    i += 1;
                }
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }

    out
}

/// Does a scrubbed, left-trimmed line begin a function definition? Recognizes
/// both `function name [()]` and `name ()` forms.
fn is_func_start(trimmed: &str) -> bool {
    if let Some(rest) = strip_word(trimmed, "function") {
        return starts_with_name(rest.trim_start());
    }
    let (name, rest) = take_ident(trimmed);
    if name.is_empty() {
        return false;
    }
    let rest = rest.trim_start();
    if let Some(r) = rest.strip_prefix('(') {
        return r.trim_start().starts_with(')');
    }
    false
}

/// Collect a function definition starting at `start`, joining continuation lines
/// until the opening `{` of the body. Returns the normalized one-line
/// declaration (body dropped) and the number of source lines consumed.
fn gather_func(lines: &[String], start: usize) -> (String, usize) {
    let mut pieces: Vec<String> = Vec::new();
    let mut consumed = 0;
    let mut found = false;

    for k in start..lines.len() {
        consumed += 1;
        let s = if k == start { lines[k].trim_start() } else { lines[k].trim() };

        if let Some(pos) = s.find('{') {
            let head = s[..pos].trim();
            if !head.is_empty() {
                pieces.push(head.to_string());
            }
            found = true;
            break;
        }

        let h = s.trim();
        if !h.is_empty() {
            pieces.push(h.to_string());
        }
        if consumed >= 64 {
            break;
        }
    }

    if !found {
        // No body brace found within the window: fall back to just the head
        // line so we never swallow unrelated code.
        let head = lines[start].trim();
        let text = tidy(&collapse_ws(head));
        return (text, 1);
    }

    let text = tidy(&collapse_ws(&pieces.join(" ")));
    (text, consumed)
}

/// If a module-level statement is a constant assignment, return its normalized
/// signature (`NAME = …`). Accepts an optional leading
/// `readonly`/`declare`/`export`/`local`/`typeset` keyword (and `declare`-style
/// `-flag` tokens), then requires an ALL-CAPS identifier immediately followed by
/// a single `=`. Lowercase names, `==` comparisons and spaced `=` are rejected.
fn constant_sig(trimmed: &str) -> Option<String> {
    let mut s = trimmed;
    let mut had_keyword = false;
    loop {
        let mut advanced = false;
        for kw in ["readonly", "export", "declare", "local", "typeset"] {
            if let Some(r) = strip_word(s, kw) {
                s = r.trim_start();
                had_keyword = true;
                advanced = true;
                break;
            }
        }
        if !advanced {
            break;
        }
    }

    if had_keyword {
        // Skip `declare`-style flag tokens such as `-r`, `-gx`.
        while s.starts_with('-') {
            let end = s.find(char::is_whitespace).unwrap_or(s.len());
            s = s[end..].trim_start();
        }
    }

    let bytes = s.as_bytes();
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
    let name = &s[..i];
    let rest = &s[i..];
    if !rest.starts_with('=') || rest.starts_with("==") {
        return None;
    }

    Some(format!("{name} = …"))
}

/// If `s` begins with the whole word `word` (followed by whitespace or end),
/// return the remainder after the word.
fn strip_word<'a>(s: &'a str, word: &str) -> Option<&'a str> {
    let rest = s.strip_prefix(word)?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(rest)
    } else {
        None
    }
}

/// Split off a leading run of identifier characters (`[A-Za-z0-9_]`).
fn take_ident(s: &str) -> (&str, &str) {
    let mut idx = 0;
    for (i, ch) in s.char_indices() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            idx = i + ch.len_utf8();
        } else {
            break;
        }
    }
    s.split_at(idx)
}

/// Does `s` start with a valid bash function-name character?
fn starts_with_name(s: &str) -> bool {
    match s.chars().next() {
        Some(c) => c == '_' || c.is_ascii_alphanumeric(),
        None => false,
    }
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

/// Tidy spacing around the parameter parentheses of a joined declaration.
fn tidy(s: &str) -> String {
    let mut t = s.to_string();
    for (from, to) in [("( ", "("), (" (", "("), (" )", ")"), ("( )", "()")] {
        t = t.replace(from, to);
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sigs(src: &str) -> Vec<Signature> {
        BashLang.extract(src)
    }

    #[test]
    fn top_level_functions() {
        let src = "foo() {\n  :\n}\n\nfunction bar {\n  :\n}\n";
        let s = sigs(src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].kind, Kind::Function);
        assert_eq!(s[0].text, "foo()");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[0].line, 1);
        assert_eq!(s[1].text, "function bar");
        assert_eq!(s[1].indent, 0);
        assert_eq!(s[1].line, 5);
    }

    #[test]
    fn nested_functions_indented() {
        let src = "outer() {\n  inner() {\n    :\n  }\n}\n";
        let s = sigs(src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].text, "outer()");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[1].text, "inner()");
        assert_eq!(s[1].indent, 1);
    }

    #[test]
    fn multiline_brace_joined() {
        let src = "baz()\n{\n  echo hi\n}\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "baz()");
        assert_eq!(s[0].indent, 0);
    }

    #[test]
    fn constants_detected_and_rejected() {
        let src = "MAX=5\nreadonly MIN=1\ndeclare -r LIMIT=9\nname=2\nCOUNT==5\n";
        let texts: Vec<String> = sigs(src)
            .iter()
            .filter(|x| x.kind == Kind::Constant)
            .map(|x| x.text.clone())
            .collect();
        assert_eq!(
            texts,
            vec!["MAX = …".to_string(), "MIN = …".to_string(), "LIMIT = …".to_string()]
        );
    }

    #[test]
    fn decl_in_comment_ignored() {
        let src = "# foo() {\nbar() {\n  :\n}\n";
        let s = sigs(src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "bar()");
    }

    #[test]
    fn decl_in_string_ignored() {
        let src = "msg='not a func() {'\nreal() {\n  :\n}\n";
        let s = sigs(src);
        let funcs: Vec<&Signature> = s.iter().filter(|x| x.kind == Kind::Function).collect();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].text, "real()");
    }
}
