//! Heuristic, dependency-free signature extractor for brace-delimited
//! C-family languages: Rust, Go, JavaScript, TypeScript, Java and C.
//!
//! Like [`super::python`], this is a forward line scanner, not a real parser.
//! It first *masks* the source — blanking out the contents of comments and
//! string/char/template/raw literals while preserving line structure — so that
//! keywords and braces hiding inside them never produce false positives. It
//! then walks the masked lines, tracks brace-nesting depth (for indentation),
//! and recognizes declarations by their leading keywords or shape. Pathological
//! inputs may be mis-handled; that is an accepted trade-off for the
//! zero-dependency goal. It never panics: all indexing is bounds-checked and
//! operates on chars so non-ASCII input is safe.

use super::Language;
use crate::signature::{Kind, Signature};

/// Which brace-language dialect a [`BraceLang`] scans.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Go,
    Js,
    Ts,
    Java,
    C,
    Cpp,
    CSharp,
    Kotlin,
    Swift,
    Php,
    Scala,
    Dart,
}

impl Lang {
    /// Do `/* */` block comments nest in this language?
    fn nests_block_comments(self) -> bool {
        matches!(
            self,
            Lang::Rust | Lang::Kotlin | Lang::Swift | Lang::Scala | Lang::Dart
        )
    }

    /// Does this language use C-style `#`-prefixed preprocessor directives?
    fn has_preprocessor(self) -> bool {
        matches!(self, Lang::C | Lang::Cpp)
    }

    /// Does this language have `"""` (and for Dart `'''`) multi-line strings?
    fn has_triple_quote(self) -> bool {
        matches!(
            self,
            Lang::Java | Lang::CSharp | Lang::Kotlin | Lang::Swift | Lang::Scala | Lang::Dart
        )
    }
}

/// A signature extractor for one of the brace-delimited [`Lang`] dialects.
pub struct BraceLang {
    pub lang: Lang,
}

/// How a gathered declaration header ended.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Term {
    /// Reached end of input / line budget without a terminator.
    None,
    /// Ended at the body-opening `{`.
    Brace,
    /// Ended at a statement-terminating `;`.
    Semi,
}

impl Language for BraceLang {
    fn extract(&self, source: &str) -> Vec<Signature> {
        let masked = mask(source, self.lang);
        let mut mvec: Vec<String> = masked.lines().map(|s| s.to_string()).collect();
        // Rust `extern "C" { … }` blocks are transparent: their `fn` items belong
        // to the enclosing scope, so blank the block braces to hoist them.
        if self.lang == Lang::Rust {
            neutralize_extern_blocks(&mut mvec);
        }
        let mlines: Vec<&str> = mvec.iter().map(|s| s.as_str()).collect();
        let olines: Vec<&str> = source.lines().collect();
        let mut out = Vec::new();
        // Stack of enclosing brace blocks. `true` marks a function/other body
        // whose local `const`/`var`/`let` declarations must be suppressed;
        // `false` marks a type/class body whose members are real declarations.
        let mut stack: Vec<bool> = Vec::new();
        let mut i = 0;

        while i < mlines.len() {
            let line = mlines[i];
            let trimmed = line.trim_start();

            if trimmed.is_empty() {
                i += 1;
                continue;
            }

            let decl_indent = stack.len();

            // Rust `macro_rules!` bodies contain template tokens (`fn $name()`),
            // not real declarations — skip the whole definition.
            if self.lang == Lang::Rust && trimmed.starts_with("macro_rules!") {
                let mut depth: i32 = 0;
                let mut seen = false;
                let mut k = i;
                while k < mlines.len() {
                    for c in mlines[k].chars() {
                        match c {
                            '{' => {
                                depth += 1;
                                seen = true;
                            }
                            '}' => depth -= 1,
                            _ => {}
                        }
                    }
                    k += 1;
                    if seen && depth <= 0 {
                        break;
                    }
                }
                i = k.max(i + 1);
                continue;
            }

            // C preprocessor directives are line-oriented; handle them here so
            // they are never glued onto the following statement by `gather`.
            if self.lang.has_preprocessor() && trimmed.starts_with('#') {
                if let Some((kind, text)) = classify_define(trimmed) {
                    out.push(Signature {
                        indent: decl_indent,
                        kind,
                        text: tidy(&collapse_ws(&text)),
                        line: i + 1,
                    });
                }
                update_stack(&mut stack, &mlines[i..=i], None);
                // A directive ending in `\` continues onto the following lines;
                // skip them so multi-line macro bodies are not scanned as
                // top-level declarations.
                let mut k = i;
                while k < mlines.len() && mlines[k].ends_with('\\') {
                    k += 1;
                }
                i = (k + 1).min(mlines.len());
                continue;
            }

            if !prefilter(trimmed, self.lang) {
                // `init {…}` (Kotlin) / `static {…}` (Java) initializer blocks open
                // a body whose local declarations are not API — tag it suppressing
                // so members inside aren't emitted.
                let tag = if is_init_block(trimmed) { Some(true) } else { None };
                update_stack(&mut stack, &mlines[i..=i], tag);
                i += 1;
                continue;
            }

            // Go grouped `const ( ... )` / `var ( ... )` blocks expand to one
            // constant per member rather than a single garbled signature.
            if self.lang == Lang::Go {
                if let Some(consumed) = go_group_block(&mlines, i, &mut out, decl_indent) {
                    i += consumed.max(1);
                    continue;
                }
            }

            let (header, consumed, term) = gather(&mlines, &olines, i);

            // Inside a type/class body (tagged `false`), a return-type-less
            // `Name(...)` is a constructor, not a call — let `classify` know.
            let in_type_body = stack.last().copied() == Some(false);
            let mut pending: Option<bool> = None;
            if let Some((kind, text)) = classify(&header, term, self.lang, in_type_body) {
                let suppress =
                    kind == Kind::Constant && stack.last().copied().unwrap_or(false);
                if !suppress {
                    out.push(Signature {
                        indent: decl_indent,
                        kind,
                        text: tidy(&collapse_ws(&text)),
                        line: i + 1,
                    });
                }
                pending = Some(kind == Kind::Function);
            }

            let consumed = consumed.max(1);
            let end = (i + consumed).min(mlines.len());
            let tag = if term == Term::Brace { pending } else { None };
            update_stack(&mut stack, &mlines[i..end], tag);
            i += consumed;
        }

        out
    }
}

/// Blank the braces of Rust `extern "ABI" { … }` blocks so the items inside are
/// hoisted to the enclosing nesting level. Operates on the masked lines (where
/// the ABI string is already blanked); brace chars are replaced by spaces so the
/// masked text stays length-aligned with the source.
fn neutralize_extern_blocks(lines: &mut [String]) {
    let mut i = 0;
    while i < lines.len() {
        if is_extern_block_opener(lines[i].trim_start()) {
            // Blank the single opening brace on this line.
            lines[i] = lines[i].replace('{', " ");
            // Find and blank the matching closing brace.
            let mut depth = 1i32;
            let mut k = i + 1;
            'scan: while k < lines.len() {
                let cs: Vec<char> = lines[k].chars().collect();
                for (ci, &c) in cs.iter().enumerate() {
                    if c == '{' {
                        depth += 1;
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            let mut nc = cs.clone();
                            nc[ci] = ' ';
                            lines[k] = nc.into_iter().collect();
                            break 'scan;
                        }
                    }
                }
                k += 1;
            }
        }
        i += 1;
    }
}

/// Is `t` (a masked, left-trimmed line) an `init {` (Kotlin) / `static {` (Java)
/// initializer-block opener, with the `{` on the same line?
fn is_init_block(t: &str) -> bool {
    for kw in ["init", "static"] {
        if let Some(r) = t.strip_prefix(kw) {
            if r.is_empty() || r.starts_with(|c: char| c.is_whitespace() || c == '{') {
                if r.trim_start().starts_with('{') {
                    return true;
                }
            }
        }
    }
    false
}

/// Is `t` (a masked, left-trimmed line) the opener of an `extern` block — i.e.
/// `extern [unsafe] "ABI" {` with the brace alone after the keyword/ABI?
fn is_extern_block_opener(t: &str) -> bool {
    let t = t.strip_prefix("pub").map(|r| r.trim_start()).unwrap_or(t);
    let t = t.strip_prefix("unsafe").map(|r| r.trim_start()).unwrap_or(t);
    let rest = match t.strip_prefix("extern") {
        Some(r) => r,
        None => return false,
    };
    if rest.chars().next().map_or(false, is_ident_byte_char) {
        return false;
    }
    rest.trim() == "{"
}

/// Advance the block stack over `lines`, pushing on `{` and popping on `}`.
/// When `body_tag` is `Some` and a net-new block was opened, tag that block.
fn update_stack(stack: &mut Vec<bool>, lines: &[&str], body_tag: Option<bool>) {
    let before = stack.len();
    for line in lines {
        for c in line.chars() {
            match c {
                '{' => stack.push(false),
                '}' => {
                    stack.pop();
                }
                _ => {}
            }
        }
    }
    if let Some(tag) = body_tag {
        if stack.len() > before {
            if let Some(top) = stack.last_mut() {
                *top = tag;
            }
        }
    }
}

/// Handle a Go grouped `const`/`var`/`type` block: `const ( A = 1\n B = 2 )` or
/// `type ( ID int64\n Name string )`. Emits one signature per member (constants
/// for const/var, type declarations for type). Returns the number of lines
/// consumed, or `None` if the line at `start` is not such a block.
fn go_group_block(
    mlines: &[&str],
    start: usize,
    out: &mut Vec<Signature>,
    indent: usize,
) -> Option<usize> {
    let first = mlines[start].trim_start();
    let (kw, after) = take_ident(first);
    if !matches!(kw, "const" | "var" | "type") {
        return None;
    }
    if !after.trim_start().starts_with('(') {
        return None;
    }
    let is_type = kw == "type";

    let mut depth: i32 = 0; // paren depth
    let mut bdepth: i32 = 0; // brace depth (skip multi-line struct/interface bodies)
    let mut consumed = 0usize;
    let mut k = start;
    while k < mlines.len() {
        consumed += 1;
        let raw = mlines[k];
        // At group level (inside the parens, outside any member body), the
        // leading identifier of a line names a member.
        if depth >= 1 && bdepth == 0 {
            let t = raw.trim_start();
            if !t.starts_with(')') {
                let (name, _) = take_ident(t);
                if !name.is_empty() && !is_control(name) {
                    let (kind, text) = if is_type {
                        // `type X Y` / alias `X = Y` / `X struct {…}` (body cut).
                        let head = match t.find('{') {
                            Some(b) => t[..b].trim_end(),
                            None => t.trim_end(),
                        };
                        (Kind::Class, format!("type {}", collapse_ws(head)))
                    } else {
                        (Kind::Constant, format!("{kw} {name} = …"))
                    };
                    out.push(Signature { indent, kind, text, line: k + 1 });
                }
            }
        }
        for c in raw.chars() {
            match c {
                '(' => depth += 1,
                ')' => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                '{' => bdepth += 1,
                '}' => {
                    if bdepth > 0 {
                        bdepth -= 1;
                    }
                }
                _ => {}
            }
        }
        if depth <= 0 {
            break;
        }
        k += 1;
        if consumed > 2000 {
            break;
        }
    }
    Some(consumed)
}

// ---------------------------------------------------------------------------
// Masking: blank out comments and literals, preserving line structure.
// ---------------------------------------------------------------------------

/// Produce a copy of `source` where every comment and string/char/raw/template
/// literal has its contents replaced by spaces (newlines preserved), so the
/// structural scan only ever sees real code. Operates on chars to stay
/// panic-free on arbitrary input.
fn mask(source: &str, lang: Lang) -> String {
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // Line comment `// ...` — blank to spaces (length preserved so the
        // masked stream stays index-aligned with the original source). In C/C++
        // a backslash immediately before the newline splices the next line into
        // the comment (phase-2 line splicing), so keep blanking across it.
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            loop {
                let mut last_was_backslash = false;
                while i < n && chars[i] != '\n' {
                    last_was_backslash = chars[i] == '\\';
                    out.push(' ');
                    i += 1;
                }
                if matches!(lang, Lang::C | Lang::Cpp) && last_was_backslash && i < n {
                    out.push('\n');
                    i += 1;
                    continue;
                }
                break;
            }
            continue;
        }
        // PHP `#[Attr]` attribute: mask the balanced `#[ … ]` region (not to end
        // of line) so an inline attribute in a parameter list doesn't blank the
        // rest of the signature. `#` alone is still a line comment.
        if lang == Lang::Php && c == '#' && i + 1 < n && chars[i + 1] == '[' {
            i = mask_php_attribute(&chars, i, &mut out);
            continue;
        }
        // PHP line comment `# ...`.
        if lang == Lang::Php && c == '#' {
            while i < n && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        // PHP heredoc/nowdoc `<<<EOT … EOT` / `<<<'EOT' … EOT`: blank the body so
        // its contents are not scanned as code.
        if lang == Lang::Php && c == '<' && i + 2 < n && chars[i + 1] == '<' && chars[i + 2] == '<' {
            if let Some(ni) = mask_php_heredoc(&chars, i, &mut out) {
                i = ni;
                continue;
            }
        }
        // Block comment `/* ... */`. In some langs these nest, tracked by depth.
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            out.push(' ');
            out.push(' ');
            i += 2;
            let mut bdepth = 1;
            while i < n && bdepth > 0 {
                if lang.nests_block_comments()
                    && chars[i] == '/'
                    && i + 1 < n
                    && chars[i + 1] == '*'
                {
                    bdepth += 1;
                    out.push(' ');
                    out.push(' ');
                    i += 2;
                    continue;
                }
                if chars[i] == '*' && i + 1 < n && chars[i + 1] == '/' {
                    bdepth -= 1;
                    out.push(' ');
                    out.push(' ');
                    i += 2;
                    continue;
                }
                out.push(if chars[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            continue;
        }

        // Rust raw string literals: r"...", r#"..."#, br"...", br#"..."#
        if lang == Lang::Rust && (c == 'r' || c == 'b') {
            if let Some(ni) = mask_rust_raw(&chars, i, &mut out) {
                i = ni;
                continue;
            }
        }

        // C++ raw string literals: R"(...)" / R"delim(...)delim" (with optional
        // L/u/U/u8 encoding prefix). Span newlines and ignore their contents.
        if lang == Lang::Cpp && c == 'R' && i + 1 < n && chars[i + 1] == '"' {
            let prev_ok = i == 0 || !is_ident_byte_char(chars[i - 1]) || is_cpp_str_prefix(&chars, i);
            if prev_ok {
                i = mask_cpp_raw(&chars, i, &mut out);
                continue;
            }
        }

        // C# interpolated-verbatim strings `$@"..."` / `@$"..."` span newlines
        // (`""` escapes the quote). The plain `@"..."` form is handled by the `@`
        // block below; these two need a dedicated check because `$` is an
        // identifier char (it defeats that block's boundary guard).
        if lang == Lang::CSharp && (c == '$' || c == '@') && i + 2 < n {
            let qpos = if c == '$' && chars[i + 1] == '@' && chars[i + 2] == '"' {
                Some(i + 2)
            } else if c == '@' && chars[i + 1] == '$' && chars[i + 2] == '"' {
                Some(i + 2)
            } else {
                None
            };
            if let Some(qpos) = qpos {
                for _ in i..qpos {
                    out.push(' ');
                }
                i = mask_verbatim_body(&chars, qpos, &mut out);
                continue;
            }
        }

        // `@` handling: annotations/decorators, C# verbatim strings, etc.
        if c == '@' && (i == 0 || !is_ident_byte_char(chars[i - 1])) {
            match lang {
                // Java's `@interface` is an annotation-type *declaration*, not an
                // annotation use — leave it intact so it can be classified.
                Lang::Java => {
                    if starts_with_word(&chars, i + 1, "interface") {
                        out.push('@');
                        i += 1;
                        continue;
                    }
                    i = mask_annotation(&chars, i, &mut out);
                    continue;
                }
                Lang::Ts | Lang::Kotlin | Lang::Swift | Lang::Dart => {
                    i = mask_annotation(&chars, i, &mut out);
                    continue;
                }
                // C#: `@"..."` is a verbatim string; `@name` is an escaped
                // identifier (left as ordinary code).
                Lang::CSharp => {
                    if i + 1 < n && chars[i + 1] == '"' {
                        i = mask_verbatim(&chars, i, &mut out);
                        continue;
                    }
                    out.push('@');
                    i += 1;
                    continue;
                }
                _ => {
                    out.push('@');
                    i += 1;
                    continue;
                }
            }
        }

        match c {
            '"' => {
                // Multi-line `"""..."""` strings (Java text blocks, Kotlin/Swift/
                // Scala/Dart raw strings, C# raw strings) span newlines.
                if lang.has_triple_quote()
                    && i + 2 < n
                    && chars[i + 1] == '"'
                    && chars[i + 2] == '"'
                {
                    i = mask_text_block(&chars, i, '"', &mut out);
                    continue;
                }
                // Rust double-quoted strings span multiple physical lines.
                let allow_nl = lang == Lang::Rust;
                i = mask_quoted(&chars, i, '"', allow_nl, true, &mut out);
                continue;
            }
            '`' if matches!(lang, Lang::Go | Lang::Js | Lang::Ts) => {
                // Go raw string (no escapes) / JS template literal (escapes): both
                // span newlines.
                let escape = matches!(lang, Lang::Js | Lang::Ts);
                i = mask_quoted(&chars, i, '`', true, escape, &mut out);
                continue;
            }
            '\'' => {
                // Dart `'''...'''` multi-line strings.
                if lang == Lang::Dart
                    && i + 2 < n
                    && chars[i + 1] == '\''
                    && chars[i + 2] == '\''
                {
                    i = mask_text_block(&chars, i, '\'', &mut out);
                    continue;
                }
                match lang {
                    // Languages where `'...'` is a (single- or multi-char) string.
                    Lang::Js | Lang::Ts | Lang::Php | Lang::Dart => {
                        i = mask_quoted(&chars, i, '\'', false, true, &mut out);
                        continue;
                    }
                    Lang::Rust => {
                        i = mask_rust_char(&chars, i, &mut out);
                        continue;
                    }
                    // Languages where `'x'` is a single char/rune literal.
                    Lang::C
                    | Lang::Cpp
                    | Lang::CSharp
                    | Lang::Java
                    | Lang::Go
                    | Lang::Kotlin
                    | Lang::Scala => {
                        i = mask_char_literal(&chars, i, &mut out);
                        continue;
                    }
                    // Swift has no single-quote literal: treat as ordinary code.
                    Lang::Swift => {
                        out.push('\'');
                        i += 1;
                        continue;
                    }
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

/// Blank a simple delimited literal starting at `start` (the opening delimiter).
/// `allow_newline` permits the literal to span lines (templates / Go raw). For
/// non-newline literals a stray unterminated quote stops at end of line.
fn mask_quoted(
    chars: &[char],
    start: usize,
    delim: char,
    allow_newline: bool,
    escape: bool,
    out: &mut String,
) -> usize {
    let n = chars.len();
    out.push(' ');
    let mut i = start + 1;
    // Normal quotes and JS template literals honor backslash escapes; Go raw
    // strings (backtick) do NOT — there a `\` is literal, so a string ending in
    // `\` must not consume the closing delimiter (`escape = false`).
    while i < n {
        let c = chars[i];
        if escape && c == '\\' && i + 1 < n {
            out.push(' ');
            // Preserve a newline that the backslash escapes so the masked
            // stream stays line-aligned with the source (e.g. Rust/C string
            // continuations `"foo \<newline>bar"`).
            out.push(if chars[i + 1] == '\n' { '\n' } else { ' ' });
            i += 2;
            continue;
        }
        if c == delim {
            out.push(' ');
            i += 1;
            break;
        }
        if c == '\n' {
            if allow_newline {
                out.push('\n');
                i += 1;
                continue;
            }
            break;
        }
        out.push(' ');
        i += 1;
    }
    i
}

/// Blank a C/Java/Go char/rune literal `'...'`, stopping at end of line if it is
/// not closed (defensive against stray quotes).
fn mask_char_literal(chars: &[char], start: usize, out: &mut String) -> usize {
    let n = chars.len();
    out.push(' ');
    let mut i = start + 1;
    while i < n {
        let c = chars[i];
        if c == '\\' && i + 1 < n {
            out.push(' ');
            out.push(' ');
            i += 2;
            continue;
        }
        if c == '\'' {
            out.push(' ');
            i += 1;
            break;
        }
        if c == '\n' {
            break;
        }
        out.push(' ');
        i += 1;
    }
    i
}

/// Handle a Rust `'`: distinguish a char literal (`'x'`, `'\n'`) from a lifetime
/// (`'a`, `'static`). Lifetimes are left as ordinary code.
fn mask_rust_char(chars: &[char], start: usize, out: &mut String) -> usize {
    let n = chars.len();
    if start + 1 < n && chars[start + 1] == '\\' {
        // Escaped char literal — consume to the closing quote.
        return mask_char_literal(chars, start, out);
    }
    if start + 2 < n && chars[start + 2] == '\'' {
        out.push(' ');
        out.push(' ');
        out.push(' ');
        return start + 3;
    }
    // Lifetime: keep the quote as ordinary punctuation.
    out.push('\'');
    start + 1
}

/// Try to mask a Rust raw string literal beginning at `start`. Returns the index
/// after the literal if one was found, else `None`.
fn mask_rust_raw(chars: &[char], start: usize, out: &mut String) -> Option<usize> {
    let n = chars.len();
    // Must start at a token boundary.
    if start > 0 && is_ident_byte_char(chars[start - 1]) {
        return None;
    }
    let mut j = start;
    if chars[j] == 'b' {
        j += 1;
    }
    if j >= n || chars[j] != 'r' {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while j < n && chars[j] == '#' {
        hashes += 1;
        j += 1;
    }
    if j >= n || chars[j] != '"' {
        return None;
    }
    j += 1; // past opening quote
    for _ in start..j {
        out.push(' ');
    }
    let mut i = j;
    while i < n {
        if chars[i] == '"' {
            let mut k = i + 1;
            let mut cnt = 0;
            while k < n && cnt < hashes && chars[k] == '#' {
                cnt += 1;
                k += 1;
            }
            if cnt == hashes {
                for _ in i..k {
                    out.push(' ');
                }
                return Some(k);
            }
        }
        out.push(if chars[i] == '\n' { '\n' } else { ' ' });
        i += 1;
    }
    Some(i)
}

/// Does the `R` at `pos` follow a valid C++ string encoding prefix (`L`, `u`,
/// `U`, `u8`) that is itself at a token boundary? Lets `u8R"..."` etc. be seen
/// as a raw string even though `8`/`u` are identifier chars.
fn is_cpp_str_prefix(chars: &[char], pos: usize) -> bool {
    // Single-letter prefix: L / u / U.
    if pos >= 1 && matches!(chars[pos - 1], 'L' | 'u' | 'U') {
        return pos < 2 || !is_ident_byte_char(chars[pos - 2]);
    }
    // Two-char prefix: u8.
    if pos >= 2 && chars[pos - 1] == '8' && chars[pos - 2] == 'u' {
        return pos < 3 || !is_ident_byte_char(chars[pos - 3]);
    }
    false
}

/// Mask a C++ raw string literal `R"delim(...)delim"` starting at the `R`
/// (`chars[start + 1]` is `"`). Spans newlines (preserved). Returns the index
/// just past the closing `"`.
fn mask_cpp_raw(chars: &[char], start: usize, out: &mut String) -> usize {
    let n = chars.len();
    out.push(' '); // R
    out.push(' '); // opening "
    let mut i = start + 2;
    // Delimiter: any chars up to the opening `(` (C++ forbids `(`, `)`,
    // whitespace and backslash in it).
    let mut delim: Vec<char> = Vec::new();
    while i < n && chars[i] != '(' {
        if chars[i] == '"' || chars[i].is_whitespace() {
            break; // malformed — give up gracefully
        }
        delim.push(chars[i]);
        out.push(' ');
        i += 1;
    }
    if i >= n || chars[i] != '(' {
        return i;
    }
    out.push(' '); // (
    i += 1;
    // Closing sequence is `)` + delim + `"`.
    let mut close: Vec<char> = Vec::with_capacity(delim.len() + 2);
    close.push(')');
    close.extend_from_slice(&delim);
    close.push('"');
    while i < n {
        if chars[i] == ')' && i + close.len() <= n && chars[i..i + close.len()] == close[..] {
            for _ in 0..close.len() {
                out.push(' ');
            }
            return i + close.len();
        }
        out.push(if chars[i] == '\n' { '\n' } else { ' ' });
        i += 1;
    }
    i
}

/// Blank a PHP 8 attribute `#[ … ]` (balanced brackets, may span lines). Returns
/// the index just past the closing `]`.
fn mask_php_attribute(chars: &[char], start: usize, out: &mut String) -> usize {
    let n = chars.len();
    out.push(' '); // #
    out.push(' '); // [
    let mut i = start + 2;
    let mut depth = 1;
    while i < n && depth > 0 {
        match chars[i] {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
        out.push(if chars[i] == '\n' { '\n' } else { ' ' });
        i += 1;
    }
    i
}

/// Blank a PHP heredoc/nowdoc body. `chars[start..]` begins with `<<<`. Returns
/// the index after the closing delimiter identifier (leaving any trailing `;`/`,`
/// as code), or `None` if this is not a valid heredoc opener.
fn mask_php_heredoc(chars: &[char], start: usize, out: &mut String) -> Option<usize> {
    let n = chars.len();
    let mut i = start + 3;
    while i < n && (chars[i] == ' ' || chars[i] == '\t') {
        i += 1;
    }
    let quote = if i < n && (chars[i] == '\'' || chars[i] == '"') {
        let q = chars[i];
        i += 1;
        Some(q)
    } else {
        None
    };
    let id0 = i;
    while i < n && (chars[i] == '_' || chars[i].is_alphanumeric()) {
        i += 1;
    }
    if i == id0 {
        return None;
    }
    let ident: Vec<char> = chars[id0..i].to_vec();
    if let Some(q) = quote {
        if i < n && chars[i] == q {
            i += 1;
        } else {
            return None;
        }
    }
    // The opener must be at end of line (only trailing whitespace allowed).
    let mut j = i;
    while j < n && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\r') {
        j += 1;
    }
    if j < n && chars[j] != '\n' {
        return None;
    }
    // Blank the opener `<<<…IDENT` and the rest of its line, keeping the newline.
    for _ in start..i {
        out.push(' ');
    }
    while i < n && chars[i] != '\n' {
        out.push(' ');
        i += 1;
    }
    if i < n {
        out.push('\n');
        i += 1;
    }
    // Blank body lines until the closing delimiter (PHP 7.3+ allows it to be
    // indented and followed by a non-identifier char).
    while i < n {
        let mut p = i;
        while p < n && (chars[p] == ' ' || chars[p] == '\t') {
            p += 1;
        }
        let is_closer = ident.iter().enumerate().all(|(k, &ic)| chars.get(p + k) == Some(&ic))
            && {
                let after = p + ident.len();
                after >= n || !(chars[after] == '_' || chars[after].is_alphanumeric())
            };
        if is_closer {
            while i < p + ident.len() {
                out.push(' ');
                i += 1;
            }
            return Some(i);
        }
        while i < n && chars[i] != '\n' {
            out.push(' ');
            i += 1;
        }
        if i < n {
            out.push('\n');
            i += 1;
        }
    }
    Some(i)
}

/// Blank a Java/TS annotation `@Name`, `@Name.Sub`, optionally `@Name(...)`.
fn mask_annotation(chars: &[char], start: usize, out: &mut String) -> usize {
    let n = chars.len();
    out.push(' ');
    let mut i = start + 1;
    while i < n && (is_ident_byte_char(chars[i]) || chars[i] == '.') {
        out.push(' ');
        i += 1;
    }
    if i < n && chars[i] == '(' {
        let mut depth = 0;
        while i < n {
            match chars[i] {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    out.push(' ');
                    i += 1;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                '\n' => {
                    out.push('\n');
                    i += 1;
                    continue;
                }
                _ => {}
            }
            out.push(' ');
            i += 1;
        }
    }
    i
}

/// Blank a triple-`delim` text block `"""..."""` / `'''...'''` (multi-line),
/// preserving newlines.
fn mask_text_block(chars: &[char], start: usize, delim: char, out: &mut String) -> usize {
    let n = chars.len();
    out.push(' ');
    out.push(' ');
    out.push(' ');
    let mut i = start + 3;
    while i < n {
        if chars[i] == delim && i + 2 < n && chars[i + 1] == delim && chars[i + 2] == delim {
            out.push(' ');
            out.push(' ');
            out.push(' ');
            return i + 3;
        }
        out.push(if chars[i] == '\n' { '\n' } else { ' ' });
        i += 1;
    }
    i
}

/// Blank a C# verbatim string `@"..."`, where `""` is an escaped quote. Spans
/// newlines, preserving them.
fn mask_verbatim(chars: &[char], start: usize, out: &mut String) -> usize {
    out.push(' '); // @
    mask_verbatim_body(chars, start + 1, out)
}

/// Mask a C# verbatim-string body starting at the opening `"` (`qpos`). Spans
/// newlines; `""` escapes a quote. Returns the index just past the closing `"`.
fn mask_verbatim_body(chars: &[char], qpos: usize, out: &mut String) -> usize {
    let n = chars.len();
    out.push(' '); // opening "
    let mut i = qpos + 1;
    while i < n {
        if chars[i] == '"' {
            if i + 1 < n && chars[i + 1] == '"' {
                out.push(' ');
                out.push(' ');
                i += 2;
                continue;
            }
            out.push(' ');
            i += 1;
            break;
        }
        out.push(if chars[i] == '\n' { '\n' } else { ' ' });
        i += 1;
    }
    i
}

/// Does the identifier starting at `pos` in `chars` equal `word` exactly (with a
/// non-identifier boundary after it)?
fn starts_with_word(chars: &[char], pos: usize, word: &str) -> bool {
    let wb: Vec<char> = word.chars().collect();
    if pos + wb.len() > chars.len() {
        return false;
    }
    for (k, wc) in wb.iter().enumerate() {
        if chars[pos + k] != *wc {
            return false;
        }
    }
    let after = pos + wb.len();
    after >= chars.len() || !is_ident_byte_char(chars[after])
}

fn is_ident_byte_char(c: char) -> bool {
    c == '_' || c == '$' || c.is_ascii_alphanumeric()
}

// ---------------------------------------------------------------------------
// Declaration gathering & classification.
// ---------------------------------------------------------------------------

/// Join a declaration header starting at line `start`, stopping at the body's
/// opening `{` or a terminating `;` found at paren-depth zero. Returns the
/// collapsed header text (terminator dropped), the number of lines consumed and
/// how it ended.
fn gather(mlines: &[&str], olines: &[&str], start: usize) -> (String, usize, Term) {
    // `paren` tracks `()`/`[]`, `angle` tracks `<>` (so multi-line generics and
    // braces inside generic bounds don't end the header prematurely).
    let mut paren: i32 = 0;
    let mut angle: i32 = 0;
    // Once a top-level `=` is seen we are in value/expression territory, where
    // `<`/`>`/`<<` are comparison/shift operators, not generic brackets.
    let mut seen_top_eq = false;
    let mut pieces: Vec<String> = Vec::new();
    let mut consumed = 0;
    let mut term = Term::None;

    let mut k = start;
    while k < mlines.len() {
        consumed += 1;
        let mchars: Vec<char> = mlines[k].chars().collect();
        let ochars: Vec<char> = if k < olines.len() {
            olines[k].chars().collect()
        } else {
            mchars.clone()
        };
        let lead = leading_ws(&mchars);

        // Source char at masked index `p` (original text is emitted into the
        // header so string/char literal contents survive masking).
        let och = |p: usize| -> char {
            if p < ochars.len() {
                ochars[p]
            } else if p < mchars.len() {
                mchars[p]
            } else {
                ' '
            }
        };

        let mut piece = String::new();
        let mut stop = false;
        let mut j = lead;
        while j < mchars.len() {
            let mc = mchars[j];
            match mc {
                // C++ operator-overload name: consume `operator` plus its symbol
                // (`<<`, `<=>`, `->`, `[]`, `()`, `==`, `=` …) as literal text so
                // the symbol's `<`/`>`/`(`/`[` are not counted as generics/parens.
                'o' if paren <= 0
                    && angle <= 0
                    && !seen_top_eq
                    && is_word_at(&mchars, j, "operator")
                    && (j == lead || !is_ident_byte_char(mchars[j - 1])) =>
                {
                    let endp = operator_token_end(&mchars, j);
                    for p in j..endp {
                        piece.push(och(p));
                    }
                    j = endp;
                }
                '(' | '[' => {
                    paren += 1;
                    piece.push(och(j));
                    j += 1;
                }
                ')' | ']' => {
                    if paren > 0 {
                        paren -= 1;
                    }
                    piece.push(och(j));
                    j += 1;
                }
                // Only treat `<` as a generic opener when it directly follows an
                // identifier / closing `>` (`Map<`, `Vec<Vec<`). Otherwise it is a
                // comparison or a symbolic operator-method name (Scala `def <(o)`),
                // which must not push the angle counter (it would never balance and
                // would swallow the rest of the file).
                '<' if paren <= 0
                    && !seen_top_eq
                    && piece
                        .chars()
                        .last()
                        .map_or(false, |c| c == '_' || c == '$' || c == '>' || c.is_alphanumeric()) =>
                {
                    angle += 1;
                    piece.push(och(j));
                    j += 1;
                }
                '>' if paren <= 0 && !seen_top_eq => {
                    if angle > 0 {
                        angle -= 1;
                    }
                    piece.push(och(j));
                    j += 1;
                }
                '=' if paren <= 0 && angle <= 0 => {
                    seen_top_eq = true;
                    piece.push(och(j));
                    j += 1;
                }
                ';' if paren <= 0 && angle <= 0 => {
                    term = Term::Semi;
                    stop = true;
                    break;
                }
                '{' if paren <= 0 && angle <= 0 => {
                    if is_type_literal_brace(&mchars, lead, j) {
                        if let Some(close) = match_brace_same_line(&mchars, j) {
                            // An inline type / object literal (e.g. `interface{}`
                            // or a return-type object) — keep it verbatim.
                            for p in j..=close {
                                piece.push(och(p));
                            }
                            j = close + 1;
                            continue;
                        }
                        // A multi-line value (`type X = {`-style) — treat the
                        // brace as the body opener and drop it.
                        term = Term::Brace;
                        stop = true;
                        break;
                    }
                    term = Term::Brace;
                    stop = true;
                    break;
                }
                _ => {
                    piece.push(och(j));
                    j += 1;
                }
            }
        }

        let piece = piece.trim_end().to_string();
        if !piece.is_empty() {
            pieces.push(piece);
        }
        if stop {
            break;
        }
        // No terminator on this line. Keep joining only while inside unbalanced
        // brackets / generics; otherwise the statement ends at the newline. This
        // keeps semicolon-less languages (Go, JS) from swallowing the following
        // declaration.
        if paren <= 0 && angle <= 0 {
            let mut p = k + 1;
            while p < mlines.len() && mlines[p].trim().is_empty() {
                p += 1;
            }
            if p < mlines.len() {
                let next = mlines[p].trim_start();
                // A generic-constraint clause (C# / Rust `where ...`) sits between
                // the header and its `{`/`;`. Consume it (so its lines don't get
                // scanned as their own declarations and its body brace is still
                // attributed correctly) but DROP it from the header text — matching
                // the convention that constraints are elided from signatures.
                if next == "where" || next.starts_with("where ") {
                    let mut q = p;
                    while q < mlines.len() {
                        let l = mlines[q].trim_start();
                        if l.is_empty() {
                            q += 1;
                        } else if l.contains('{') {
                            // Body opener — may be on the where line itself
                            // (`where T : Any {`). Leave line q unconsumed so its
                            // `{` is counted by the next iteration.
                            term = Term::Brace;
                            break;
                        } else if l.contains(';') {
                            term = Term::Semi;
                            q += 1;
                            break;
                        } else {
                            q += 1;
                        }
                    }
                    consumed += q - k - 1; // header line already counted
                    break;
                }
                if next.starts_with('{') {
                    term = Term::Brace;
                } else if next.starts_with(';') {
                    term = Term::Semi;
                }
            }
            break;
        }
        if consumed > 200 {
            break;
        }
        k += 1;
    }

    (collapse_ws(&join_pieces(&pieces)), consumed, term)
}

/// Index of the first non-whitespace char in `chars`.
fn leading_ws(chars: &[char]) -> usize {
    let mut i = 0;
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    i
}

/// Is the `{` at index `j` an inline type/value literal (kept) rather than a
/// body opener (dropped)? Heuristic: a `{` directly following `=`, `:`, `|` or
/// `,`, or adjacent to a `struct`/`interface` keyword (Go anonymous types).
fn is_type_literal_brace(mchars: &[char], lead: usize, j: usize) -> bool {
    // Find the last non-whitespace char before `{` (whitespace between the token
    // and the `{` is allowed, e.g. `default {…}`).
    let mut p = j;
    while p > lead {
        p -= 1;
        if mchars[p].is_whitespace() {
            continue;
        }
        if matches!(mchars[p], '=' | ':' | '|' | ',' | '&') {
            return true;
        }
        // `p` is the last char of the preceding token; extract the whole word.
        if mchars[p].is_alphanumeric() || mchars[p] == '_' {
            let mut s = p + 1;
            while s > lead && (mchars[s - 1].is_alphanumeric() || mchars[s - 1] == '_') {
                s -= 1;
            }
            let word: String = mchars[s..p + 1].iter().collect();
            // `interface`/`struct` introduce an anonymous type literal; `default`
            // introduces a Java annotation element's array value (`elem() default {…}`).
            if matches!(word.as_str(), "interface" | "struct" | "default") {
                return true;
            }
        }
        break;
    }
    false
}

/// Find the `}` matching the `{` at `open` on the same line, or `None`.
fn match_brace_same_line(mchars: &[char], open: usize) -> Option<usize> {
    let mut d: i32 = 0;
    let mut p = open;
    while p < mchars.len() {
        match mchars[p] {
            '{' => d += 1,
            '}' => {
                d -= 1;
                if d == 0 {
                    return Some(p);
                }
            }
            _ => {}
        }
        p += 1;
    }
    None
}

/// Join header pieces with single spaces, except across bracket boundaries so a
/// multi-line `<...>` / `(...)` does not introduce spurious spaces.
fn join_pieces(pieces: &[String]) -> String {
    let mut out = String::new();
    for p in pieces {
        if out.is_empty() {
            out.push_str(p);
            continue;
        }
        let prev = out.chars().last().unwrap_or(' ');
        let cur = p.chars().next().unwrap_or(' ');
        let no_space = matches!(prev, '<' | '(' | '[') || matches!(cur, '>' | ')' | ']');
        if !no_space {
            out.push(' ');
        }
        out.push_str(p);
    }
    out
}

/// Cheap test on the first line of a potential declaration: is it worth
/// gathering and classifying? Excludes control-flow statements so their bodies
/// are never mistaken for declarations.
fn prefilter(t: &str, lang: Lang) -> bool {
    if t.is_empty() {
        return false;
    }
    if lang.has_preprocessor() && t.starts_with('#') {
        return t[1..].trim_start().starts_with("define");
    }

    let (_, core) = split_mods(t, lang);
    let (kw, _) = take_ident(core);

    // Java annotation-type declaration: `@interface Name`.
    if lang == Lang::Java && core.trim_start().starts_with("@interface") {
        return true;
    }

    if !kw.is_empty() {
        if is_control(kw) {
            return false;
        }
        if class_keywords(lang).contains(&kw) {
            return true;
        }
        match lang {
            Lang::Rust => {
                if matches!(kw, "fn" | "const" | "static") {
                    return true;
                }
            }
            Lang::Go => {
                if matches!(kw, "func" | "const" | "var" | "type") {
                    return true;
                }
            }
            Lang::Js | Lang::Ts => {
                if matches!(kw, "function" | "const" | "let" | "var") {
                    return true;
                }
            }
            Lang::C | Lang::Cpp => {
                if matches!(kw, "const" | "constexpr" | "static") {
                    return true;
                }
            }
            Lang::Kotlin => {
                if matches!(kw, "fun" | "val" | "var") {
                    return true;
                }
            }
            Lang::Swift => {
                if matches!(kw, "func" | "let" | "var" | "subscript" | "init") {
                    return true;
                }
            }
            Lang::Php => {
                if matches!(kw, "function" | "const") {
                    return true;
                }
            }
            Lang::Scala => {
                if matches!(kw, "def" | "val" | "var") {
                    return true;
                }
            }
            Lang::Dart => {
                if matches!(kw, "const" | "final" | "var") {
                    return true;
                }
                // Getters have no `(`, so the callable-shape check below misses them.
                if dart_getter(core).is_some() {
                    return true;
                }
            }
            Lang::CSharp => {
                if matches!(kw, "const") {
                    return true;
                }
            }
            Lang::Java => {}
        }
    }

    // A Java/C# field / constant: `... NAME = value`. The value may itself
    // contain a method call, so detect by a top-level `=` before any `(`.
    // `classify` then keeps only the genuinely constant ones.
    if matches!(lang, Lang::Java | Lang::CSharp) {
        if let Some(eq) = find_top_eq(t) {
            let pp = t.find('(').unwrap_or(usize::MAX);
            if eq < pp {
                return true;
            }
        }
    }

    // Generic callable shape `name(` for languages without a function keyword.
    if matches!(
        lang,
        Lang::C | Lang::Cpp | Lang::CSharp | Lang::Java | Lang::Js | Lang::Ts | Lang::Dart
    ) {
        if let Some(idx) = t.find('(') {
            let head = callable_head(&t[..idx]);
            let name = trailing_ident(head);
            if !name.is_empty() {
                // Reject a bare control-flow statement (`if (...)`), but NOT a
                // method whose name merely looks like a keyword (`Factory of(...)`,
                // `T with(...)`) — those carry a return type / modifier prefix.
                let prefix_empty = head[..head.len() - name.len()].trim().is_empty();
                if !(is_control(name) && prefix_empty) {
                    return true;
                }
            }
            // C++/C#/Dart operator overloads have a symbol name (`operator+`,
            // `operator[]`, `operator ==`) that `trailing_ident` cannot read.
            if matches!(lang, Lang::Cpp | Lang::CSharp | Lang::Dart)
                && operator_keyword_start(&t[..idx]).is_some()
            {
                return true;
            }
            // JS/TS computed method name `[expr]()` ends in `]`, not an identifier.
            if matches!(lang, Lang::Js | Lang::Ts) && head.trim_end().ends_with(']') {
                return true;
            }
            // TS optional method/property signature `name?()` ends in `?`.
            if lang == Lang::Ts && head.trim_end().ends_with('?') {
                return true;
            }
        }
    }

    false
}

/// Decide what kind of declaration a gathered header is, returning the kind and
/// its normalized text (body already removed by `gather`).
fn classify(header: &str, term: Term, lang: Lang, in_type_body: bool) -> Option<(Kind, String)> {
    let h = header.trim();
    if h.is_empty() {
        return None;
    }

    let (mods, core) = split_mods(h, lang);
    let (kw, rest) = take_ident(core);

    // Java annotation-type declaration: `@interface Name`.
    if lang == Lang::Java && core.trim_start().starts_with("@interface") {
        return Some((Kind::Class, class_text(h, lang)));
    }

    // C/C++ struct/union/enum/class/namespace: a real definition has a body (or
    // is a `typedef`); a bare `struct T *field;` is a type *use* (field), not a
    // declaration.
    if matches!(lang, Lang::C | Lang::Cpp) && class_keywords(lang).contains(&kw) {
        if h.contains('(') && looks_like_function(h, lang, term, in_type_body) {
            let text = if lang == Lang::Cpp { cpp_fn_text(h) } else { fn_text(h) };
            return Some((Kind::Function, text));
        }
        if kw == "typedef" || term == Term::Brace {
            return Some((Kind::Class, class_text(h, lang)));
        }
        return None;
    }

    // Type / class-like declarations.
    if class_keywords(lang).contains(&kw) {
        return Some((Kind::Class, class_text(h, lang)));
    }

    match lang {
        Lang::Rust => {
            if kw == "const" || kw == "static" {
                // Skip any intervening keywords (`mut`, `unsafe`, `async`,
                // `extern "ABI"`); a following `fn` makes this a function
                // (`const fn`, `const unsafe fn`), otherwise it is a constant.
                let mut r = rest.trim_start();
                loop {
                    let (w, after) = take_ident(r);
                    if w == "fn" {
                        return Some((Kind::Function, h.to_string()));
                    }
                    if matches!(w, "mut" | "unsafe" | "async" | "extern") {
                        r = after.trim_start();
                        if w == "extern" {
                            if let Some(q) = r.strip_prefix('"') {
                                if let Some(e) = q.find('"') {
                                    r = q[e + 1..].trim_start();
                                }
                            }
                        }
                        continue;
                    }
                    break;
                }
                let (n2, _) = take_ident(r);
                if n2.is_empty() {
                    return None; // bare `const` / `static` with no name
                }
                // A value-less associated const / extern static (`const BAR:
                // usize;`) has no initializer to elide — show it verbatim.
                if find_top_eq(h).is_none() {
                    return Some((Kind::Constant, h.trim().to_string()));
                }
                return Some((Kind::Constant, const_text(h)));
            }
            if kw == "fn" {
                return Some((Kind::Function, h.to_string()));
            }
        }
        Lang::Go => {
            if kw == "const" || kw == "var" {
                let (n2, _) = take_ident(rest.trim_start());
                if n2.is_empty() {
                    return None;
                }
                return Some((Kind::Constant, const_text(h)));
            }
            if kw == "func" {
                return Some((Kind::Function, h.to_string()));
            }
        }
        Lang::Js | Lang::Ts => {
            if matches!(kw, "const" | "let" | "var") {
                let r = rest.trim_start();
                if r.is_empty() {
                    return None; // bare keyword / destructuring with no name
                }
                // TypeScript `const enum Foo` is a type, not a constant.
                if lang == Lang::Ts {
                    let (n2, _) = take_ident(r);
                    if class_keywords(Lang::Ts).contains(&n2) {
                        return Some((Kind::Class, class_text(h, lang)));
                    }
                }
                // Only an arrow whose `=>` is in the *value* is a function.
                if arrow_in_value(h) {
                    return Some((Kind::Function, h.to_string()));
                }
                return Some((Kind::Constant, const_text(h)));
            }
            if kw == "function" {
                return Some((Kind::Function, h.to_string()));
            }
        }
        Lang::C | Lang::Cpp => {
            if matches!(kw, "const" | "constexpr" | "static") {
                if looks_like_function(h, lang, term, in_type_body) {
                    return Some((Kind::Function, h.to_string()));
                }
                // `static` alone (no `const`/`constexpr`) is mutable — not a
                // constant. A `constexpr` is always a constant.
                let is_const = contains_word(h, "const") || contains_word(h, "constexpr");
                if is_const && h.contains('=') {
                    return Some((Kind::Constant, const_text(h)));
                }
                return None;
            }
        }
        Lang::Java => {
            if mods.iter().any(|m| *m == "final") {
                if let Some(eq) = find_top_eq(h) {
                    let pp = h.find('(').unwrap_or(usize::MAX);
                    if eq < pp {
                        return Some((Kind::Constant, const_text(h)));
                    }
                }
            }
        }
        Lang::Kotlin => {
            if kw == "fun" {
                return Some((Kind::Function, fn_text(h)));
            }
            if matches!(kw, "val" | "var") {
                let (n2, _) = take_ident(rest.trim_start());
                if n2.is_empty() {
                    return None;
                }
                // A property with no initializer (a custom getter follows on the
                // next line) must not get a spurious ` = …`.
                if find_top_eq(h).is_none() {
                    return Some((Kind::Constant, h.trim().to_string()));
                }
                return Some((Kind::Constant, const_text(h)));
            }
        }
        Lang::Swift => {
            if matches!(kw, "func" | "init" | "subscript") {
                return Some((Kind::Function, h.to_string()));
            }
            if matches!(kw, "let" | "var") {
                let (n2, _) = take_ident(rest.trim_start());
                if n2.is_empty() {
                    return None;
                }
                return Some((Kind::Constant, const_text(h)));
            }
        }
        Lang::Scala => {
            if kw == "def" {
                return Some((Kind::Function, scala_def_text(h)));
            }
            if matches!(kw, "val" | "var") {
                let (n2, _) = take_ident(rest.trim_start());
                if n2.is_empty() {
                    return None;
                }
                return Some((Kind::Constant, const_text(h)));
            }
        }
        Lang::Php => {
            if kw == "function" {
                return Some((Kind::Function, h.to_string()));
            }
            if kw == "const" {
                let (n2, _) = take_ident(rest.trim_start());
                if n2.is_empty() {
                    return None;
                }
                return Some((Kind::Constant, const_text(h)));
            }
        }
        Lang::Dart => {
            if matches!(kw, "const" | "final" | "var") {
                let (n2, after) = take_ident(rest.trim_start());
                // A typed `final Foo bar = …` keeps the name as the *last*
                // identifier before `=`; but a bare `const x = …` names directly.
                if n2.is_empty() {
                    return None;
                }
                // `final` / `const` may modify a function-shaped member; if there
                // is a top-level `=` it is a constant, otherwise fall through.
                let _ = after;
                if find_top_eq(h).is_some() || term != Term::Brace {
                    return Some((Kind::Constant, const_text(h)));
                }
            }
            // Getter: `[Type] get name => …` / `{ … }`. No parameter list, so the
            // generic callable check never fires; classify it here as a function.
            if dart_getter(core).is_some() {
                return Some((Kind::Function, fn_text(h)));
            }
        }
        Lang::CSharp => {
            if kw == "const" {
                let (n2, _) = take_ident(rest.trim_start());
                // `const int Max = 1` — name is the *last* ident before `=`.
                if n2.is_empty() {
                    return None;
                }
                return Some((Kind::Constant, const_text(h)));
            }
            // `static readonly` fields are C#'s idiomatic constants.
            if mods.iter().any(|m| *m == "readonly") {
                if let Some(eq) = find_top_eq(h) {
                    let pp = h.find('(').unwrap_or(usize::MAX);
                    if eq < pp {
                        return Some((Kind::Constant, const_text(h)));
                    }
                }
            }
        }
    }

    if matches!(
        lang,
        Lang::C | Lang::Cpp | Lang::CSharp | Lang::Java | Lang::Js | Lang::Ts | Lang::Dart
    ) && looks_like_function(h, lang, term, in_type_body)
    {
        let text = if lang == Lang::Cpp { cpp_fn_text(h) } else { fn_text(h) };
        return Some((Kind::Function, text));
    }

    None
}

/// Scala `def` text: drop a trailing `= body` / `= {` so only the header shows.
fn scala_def_text(h: &str) -> String {
    if let Some(eq) = find_top_eq_scala(h) {
        h[..eq].trim_end().to_string()
    } else {
        h.trim().to_string()
    }
}

/// Normalized text for a class/type declaration: strip a trailing dangling `=`
/// (left when a multi-line object-literal value was dropped) and, for Java, a
/// `permits ...` clause.
fn class_text(h: &str, lang: Lang) -> String {
    let mut s = h.trim().to_string();
    if lang == Lang::Java {
        if let Some(p) = s.find(" permits ") {
            s.truncate(p);
        }
    }
    let trimmed = s.trim_end();
    let trimmed = trimmed.strip_suffix('=').unwrap_or(trimmed).trim_end();
    trimmed.to_string()
}

/// Does the assignment value (after the top-level `=`) contain an arrow `=>`?
fn arrow_in_value(h: &str) -> bool {
    match find_top_eq(h) {
        Some(eq) => h[eq..].contains("=>"),
        None => h.contains("=>"),
    }
}

/// Is `word` present in `h` as a whole word?
fn contains_word(h: &str, word: &str) -> bool {
    let mut rest = h;
    while let Some(p) = rest.find(word) {
        let before_ok = p == 0
            || !rest[..p]
                .chars()
                .next_back()
                .map(is_ident_byte_char)
                .unwrap_or(false);
        let after = &rest[p + word.len()..];
        let after_ok = !after.chars().next().map(is_ident_byte_char).unwrap_or(false);
        if before_ok && after_ok {
            return true;
        }
        rest = &rest[p + word.len()..];
    }
    false
}

/// Classify a C `#define` line as a constant.
fn classify_define(line: &str) -> Option<(Kind, String)> {
    let hs = line.trim_start().strip_prefix('#')?.trim_start();
    let rest = hs.strip_prefix("define")?.trim_start();
    let (name, after) = take_ident(rest);
    if name.is_empty() {
        return None;
    }
    let mut decl = format!("#define {name}");
    if after.starts_with('(') {
        if let Some(end) = matched_paren_end(after) {
            decl.push_str(&after[..end]);
        }
    }
    decl.push_str(" …");
    Some((Kind::Constant, decl))
}

/// Does `h` have the shape of a function/method definition or declaration?
fn looks_like_function(h: &str, lang: Lang, term: Term, in_type_body: bool) -> bool {
    let idx = match h.find('(') {
        Some(i) => i,
        None => return false,
    };
    let before = h[..idx].trim_end();
    let head = callable_head(before);

    // C++/C#/Dart operator functions: `R operator@(...)`, `operator T(...)`. The
    // name is the `operator` token — possibly a symbol (`operator+`, `operator[]`,
    // `operator ==`) that `trailing_ident` cannot read — so detect the keyword and
    // accept it directly, rejecting obvious expression/statement contexts.
    if matches!(lang, Lang::Cpp | Lang::CSharp | Lang::Dart) {
        if let Some(op_start) = operator_keyword_start(before) {
            if term != Term::Brace && term != Term::Semi {
                return false;
            }
            let pre = before[..op_start].trim();
            return !pre.ends_with('.')
                && !is_control(pre.split_whitespace().next().unwrap_or(""))
                && !pre.contains(|c| matches!(c, '=' | '{' | '}' | ';' | '?'));
        }
    }

    // TS optional member `name?()` — the `?` is not part of the name; read the
    // name (and prefix) from the head with a trailing `?` removed.
    let name_head = if lang == Lang::Ts {
        head.trim_end().strip_suffix('?').unwrap_or(head)
    } else {
        head
    };
    let name = trailing_ident(name_head);
    if name.is_empty() {
        // JS/TS computed method name `[expr]()` has no trailing identifier; accept
        // it as a `{`-bodied method.
        if matches!(lang, Lang::Js | Lang::Ts) && head.trim_end().ends_with(']') {
            return term == Term::Brace;
        }
        return false;
    }
    let prefix = name_head[..name_head.len() - name.len()].trim();
    // A control keyword as the bare callable name is a statement (`if (...)`);
    // a method named like one (`Factory of(...)`) has a return-type prefix.
    if is_control(name) && prefix.is_empty() {
        return false;
    }
    // Member access (`obj.method(`) is a call expression, not a declaration.
    if prefix.ends_with('.') {
        return false;
    }
    // A statement like `return foo()` / `await bar()` is a call, not a decl.
    if is_control(prefix.split_whitespace().next().unwrap_or("")) {
        return false;
    }
    // Reject expression/statement contexts. `.` and `,` are allowed inside
    // generic type arguments (Java `Map<String, Object>`, `java.util.List<E>`).
    let mut angle: i32 = 0;
    for ch in prefix.chars() {
        match ch {
            '<' => angle += 1,
            '>' => {
                if angle > 0 {
                    angle -= 1;
                }
            }
            '=' | '(' | ')' | '{' | '}' | ';' | '?' | '@' if angle == 0 => return false,
            _ => {}
        }
    }
    match lang {
        Lang::Js => term == Term::Brace,
        // TypeScript method overload signatures end with `;` and carry a return
        // type annotation; accept those alongside `{`-bodied methods.
        Lang::Ts => term == Term::Brace || (term == Term::Semi && has_return_annotation(h)),
        // Keyword-less, type-prefixed callables (definitions and prototypes).
        // A C++ member with no return type inside a class/struct body is a
        // constructor (`Foo(int)`), so allow an empty prefix there.
        Lang::C | Lang::Cpp | Lang::CSharp | Lang::Java | Lang::Dart => {
            let prefix_ok = !prefix.is_empty() || (lang == Lang::Cpp && in_type_body);
            prefix_ok && (term == Term::Brace || term == Term::Semi)
        }
        _ => false,
    }
}

/// Does `h` carry a `:` return-type annotation after its parameter list?
fn has_return_annotation(h: &str) -> bool {
    h.rfind(')').map_or(false, |p| h[p..].contains(':'))
}

/// Build a constant's normalized text, eliding the value as `= …`.
fn const_text(h: &str) -> String {
    if let Some(eq) = find_top_eq(h) {
        format!("{} = …", h[..eq].trim_end().trim())
    } else {
        format!("{} = …", h.trim())
    }
}

/// Class/type-introducing keywords per language.
fn class_keywords(lang: Lang) -> &'static [&'static str] {
    match lang {
        Lang::Rust => &["struct", "enum", "trait", "union", "type"],
        Lang::Go => &["type"],
        Lang::Js => &["class"],
        Lang::Ts => &["class", "interface", "type", "enum", "namespace"],
        Lang::Java => &["class", "interface", "enum", "record"],
        Lang::C => &["struct", "union", "enum", "typedef"],
        Lang::Cpp => &["struct", "union", "enum", "typedef", "class", "namespace"],
        Lang::CSharp => &[
            "class",
            "struct",
            "interface",
            "enum",
            "record",
            "namespace",
            "delegate",
        ],
        // Kotlin class-introducers; `data`/`sealed`/`enum`/`annotation`/`inner`
        // are handled as modifiers in `split_mods` (`enum class`, `data class`).
        Lang::Kotlin => &["class", "interface", "object", "typealias"],
        Lang::Swift => &[
            "class",
            "struct",
            "enum",
            "protocol",
            "extension",
            "typealias",
            "actor",
        ],
        Lang::Php => &["class", "interface", "trait", "enum"],
        // Scala: `case class` / `case object` handled via the `case` modifier.
        Lang::Scala => &["class", "trait", "object", "type", "enum"],
        Lang::Dart => &["class", "mixin", "enum", "extension", "typedef"],
    }
}

/// Strip leading modifier keywords from `s`, returning them and the remainder.
fn split_mods<'a>(s: &'a str, lang: Lang) -> (Vec<&'a str>, &'a str) {
    let mut rest = s.trim_start();
    let mut mods = Vec::new();
    loop {
        if rest.starts_with("pub(") {
            if let Some(end) = rest.find(')') {
                mods.push(&rest[..end + 1]);
                rest = rest[end + 1..].trim_start();
                continue;
            }
        }
        // Java `non-sealed` contains a hyphen, which `take_ident` would split.
        if lang == Lang::Java && rest.starts_with("non-sealed") {
            mods.push(&rest[..10]);
            rest = rest[10..].trim_start();
            continue;
        }
        // Scala access qualifiers `private[X]` / `protected[X]` / `private[this]`.
        if lang == Lang::Scala {
            let qual = ["private", "protected"].iter().find_map(|kw| {
                let after = rest.strip_prefix(kw)?;
                let a = after.trim_start();
                if !a.starts_with('[') {
                    return None;
                }
                let close_rel = a.find(']')?;
                Some(rest.len() - a.len() + close_rel + 1)
            });
            if let Some(end) = qual {
                mods.push(rest[..end].trim_end());
                rest = rest[end..].trim_start();
                continue;
            }
        }
        // Dart 3 class modifiers `base`/`interface`/`final` precede a class
        // keyword (or another such modifier). `final` is dual-use (it also marks
        // a constant), so only treat it as a modifier when a class-introducer
        // actually follows.
        if lang == Lang::Dart {
            let (w, after) = take_ident(rest);
            if matches!(w, "base" | "interface" | "final") {
                let (nw, _) = take_ident(after.trim_start());
                if matches!(
                    nw,
                    "class" | "mixin" | "enum" | "extension" | "base" | "interface" | "final"
                        | "sealed" | "abstract"
                ) {
                    mods.push(w);
                    rest = after.trim_start();
                    continue;
                }
            }
        }
        // Rust/C++ `extern "ABI"` prefix: consume the optional ABI string with
        // the keyword so the following `fn` is recognized.
        if matches!(lang, Lang::Rust | Lang::Cpp) && rest.starts_with("extern") {
            let tail = &rest["extern".len()..];
            if tail.chars().next().map_or(true, |c| !is_ident_byte_char(c)) {
                let t2 = tail.trim_start();
                if t2.starts_with('"') {
                    if let Some(close) = t2[1..].find('"') {
                        let abi_end = rest.len() - t2.len() + 1 + close + 1;
                        mods.push(rest[..abi_end].trim_end());
                        rest = rest[abi_end..].trim_start();
                        continue;
                    }
                }
            }
        }
        // C++ `template <...>` prefix: skip the keyword and its angle clause.
        if lang == Lang::Cpp && rest.starts_with("template") {
            let after = rest["template".len()..].trim_start();
            if after.starts_with('<') {
                if let Some(end) = matched_angle_end(after) {
                    mods.push(&rest[..rest.len() - after.len()]);
                    rest = after[end..].trim_start();
                    continue;
                }
            }
        }
        let (w, r) = take_ident(rest);
        if !w.is_empty() && is_modifier(w, lang) {
            mods.push(w);
            rest = r.trim_start();
        } else {
            break;
        }
    }
    (mods, rest)
}

fn is_modifier(w: &str, lang: Lang) -> bool {
    let common = matches!(
        w,
        "pub" | "public"
            | "private"
            | "protected"
            | "abstract"
            | "export"
            | "default"
            | "override"
            | "virtual"
            | "inline"
            | "synchronized"
            | "native"
            | "transient"
            | "volatile"
            | "strictfp"
            | "unsafe"
            | "extern"
            | "async"
    );
    let stat = matches!(
        lang,
        Lang::Java
            | Lang::Js
            | Lang::Ts
            | Lang::Cpp
            | Lang::CSharp
            | Lang::Kotlin
            | Lang::Swift
            | Lang::Php
            | Lang::Dart
    ) && w == "static";
    let fin = lang == Lang::Java && matches!(w, "final" | "sealed");
    let lang_specific = match lang {
        // `constexpr` is intentionally *not* listed: it must reach the const
        // classifier so `constexpr T X = …` is recognized as a constant.
        Lang::Cpp => matches!(w, "explicit" | "friend" | "mutable" | "thread_local"),
        Lang::CSharp => matches!(
            w,
            "internal"
                | "sealed"
                | "partial"
                | "readonly"
                | "new"
                | "explicit"
                | "implicit"
                | "volatile"
                | "unsafe"
                | "ref"
        ),
        // `enum`/`data`/`sealed`/`annotation`/`inner`/`value`/`companion` precede
        // a Kotlin `class`/`object`; `open`/`suspend`/etc. precede members.
        Lang::Kotlin => matches!(
            w,
            "open"
                | "data"
                | "sealed"
                | "enum"
                | "annotation"
                | "inner"
                | "value"
                | "companion"
                | "suspend"
                | "internal"
                | "lateinit"
                | "tailrec"
                | "operator"
                | "infix"
                | "external"
                | "const"
                | "actual"
                | "expect"
                | "final"
        ),
        Lang::Swift => matches!(
            w,
            "open"
                | "final"
                | "fileprivate"
                | "internal"
                | "convenience"
                | "required"
                | "lazy"
                | "weak"
                | "unowned"
                | "mutating"
                | "nonmutating"
                | "dynamic"
                | "indirect"
                | "optional"
                | "discardableresult"
                | "prefix"
                | "postfix"
                | "nonisolated"
        ),
        Lang::Scala => matches!(
            w,
            "case" | "sealed" | "implicit" | "lazy" | "final"
        ),
        Lang::Php => matches!(w, "final" | "readonly"),
        Lang::Ts => matches!(w, "declare" | "readonly"),
        Lang::Dart => matches!(
            w,
            "external" | "factory" | "covariant" | "late" | "abstract" | "sealed"
        ),
        _ => false,
    };
    common || stat || fin || lang_specific
}

fn is_control(w: &str) -> bool {
    matches!(
        w,
        "if" | "else"
            | "for"
            | "while"
            | "switch"
            | "case"
            | "catch"
            | "do"
            | "return"
            | "goto"
            | "sizeof"
            | "typeof"
            | "throw"
            | "throws"
            | "new"
            | "delete"
            | "await"
            | "yield"
            | "match"
            | "loop"
            | "break"
            | "continue"
            | "defer"
            | "go"
            | "select"
            | "range"
            | "in"
            | "of"
            | "with"
            | "try"
            | "finally"
            | "instanceof"
    )
}

/// Take a leading identifier from `s` (after trimming), returning it and the
/// rest of the string. Unicode-aware so non-ASCII identifiers (e.g. `ÜBER`,
/// `話す`) are kept whole rather than read as empty.
fn take_ident(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        if ch == '_' || ch == '$' || ch.is_alphanumeric() {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    (&s[..end], &s[end..])
}

/// The portion of a pre-`(` header up to (and excluding) a trailing generic
/// clause, so the method name in `name<T>(` can be found. If `before` does not
/// end in a balanced `<...>`, it is returned unchanged.
fn callable_head(before: &str) -> &str {
    let before = before.trim_end();
    if before.ends_with('>') {
        if let Some(open) = angle_open_from_end(before) {
            return before[..open].trim_end();
        }
    }
    before
}

/// Given `s` ending in `>`, the byte index of the matching `<` (depth-balanced),
/// scanning from the end. `None` if unbalanced.
fn angle_open_from_end(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (idx, ch) in s.char_indices().rev() {
        match ch {
            '>' => depth += 1,
            '<' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

/// A function header with any expression body dropped: cut at the first
/// top-level `=` (which also covers a `=>` arrow body, since it starts with
/// `=`). Default parameter `=`s live inside parens and are not top-level.
fn fn_text(h: &str) -> String {
    match find_top_eq(h) {
        Some(eq) => h[..eq].trim_end().to_string(),
        None => h.trim().to_string(),
    }
}

/// C++ function header text with any body specifier dropped. Like `fn_text` but
/// only cuts at a top-level `=` that appears *after* the parameter list, so a
/// copy/move-assignment `operator=` (whose `=` precedes the params) is preserved
/// while `= default` / `= delete` / `= 0` are still removed.
fn cpp_fn_text(h: &str) -> String {
    if let Some(open) = h.find('(') {
        if let Some(rel_close) = matched_paren_end(&h[open..]) {
            let after = open + rel_close;
            if let Some(eq_rel) = find_top_eq(&h[after..]) {
                return h[..after + eq_rel].trim_end().to_string();
            }
            return h.trim().to_string();
        }
    }
    fn_text(h)
}

/// If `core` is a Dart getter (`[Type] get <name> => …` / `{ … }` / `;`), return
/// the getter name. Getters have no parameter list, so the normal `name(` callable
/// detection misses them. The `get` keyword must be whole-word and directly
/// followed by an identifier.
fn dart_getter(core: &str) -> Option<&str> {
    let b = core.as_bytes();
    let mut i = 0;
    while let Some(rel) = core[i..].find("get") {
        let pos = i + rel;
        let before_ok = pos == 0 || !is_ident_byte_char(b[pos - 1] as char);
        let after = pos + 3;
        if before_ok && after < b.len() && (b[after] == b' ' || b[after] == b'\t') {
            let (name, _) = take_ident(core[after..].trim_start());
            if !name.is_empty() && name != "get" {
                return Some(name);
            }
        }
        i = pos + 3;
    }
    None
}

/// Does `chars[pos..]` begin with the whole word `word`, bounded after by a
/// non-identifier char? (Caller checks the boundary before `pos`.)
fn is_word_at(chars: &[char], pos: usize, word: &str) -> bool {
    let wl = word.chars().count();
    if pos + wl > chars.len() {
        return false;
    }
    if !word.chars().enumerate().all(|(k, wc)| chars[pos + k] == wc) {
        return false;
    }
    let after = pos + wl;
    after >= chars.len() || !is_ident_byte_char(chars[after])
}

/// Given `chars[start..]` beginning with the word `operator`, return the index
/// just past the full operator name (the keyword plus its symbol/conversion
/// token: `operator+`, `operator<<`, `operator<=>`, `operator[]`, `operator()`,
/// `operator bool`, `operator new[]`). Stops before the parameter list.
fn operator_token_end(chars: &[char], start: usize) -> usize {
    let n = chars.len();
    let mut i = start + "operator".len();
    while i < n && chars[i].is_whitespace() {
        i += 1;
    }
    if i >= n {
        return i;
    }
    let c = chars[i];
    // `operator()` / `operator[]`: the bracket pair is the name, not the params.
    if (c == '(' && i + 1 < n && chars[i + 1] == ')')
        || (c == '[' && i + 1 < n && chars[i + 1] == ']')
    {
        return i + 2;
    }
    // Conversion / named operator (`operator bool`, `operator new`, `delete`).
    if c == '_' || c.is_alphabetic() {
        let mut j = i;
        while j < n && (chars[j] == '_' || chars[j].is_alphanumeric()) {
            j += 1;
        }
        if j + 1 < n && chars[j] == '[' && chars[j + 1] == ']' {
            j += 2;
        }
        return j;
    }
    // Symbol operator: a run of operator-punctuation characters.
    let mut j = i;
    while j < n
        && matches!(
            chars[j],
            '+' | '-' | '*' | '/' | '%' | '^' | '&' | '|' | '~' | '!' | '=' | '<' | '>' | ','
        )
    {
        j += 1;
    }
    if j == i {
        i
    } else {
        j
    }
}

/// Byte index where a C++ `operator` keyword starts in `s`, if `s` contains the
/// whole word `operator` (bounded by non-identifier chars). Used to recognize
/// operator-overload functions whose name is a symbol (`operator+`, `operator[]`).
fn operator_keyword_start(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 0;
    while let Some(rel) = s[i..].find("operator") {
        let pos = i + rel;
        let before_ok = pos == 0 || !is_ident_byte_char(b[pos - 1] as char);
        let after = pos + "operator".len();
        let after_ok = after >= b.len() || !is_ident_byte_char(b[after] as char);
        if before_ok && after_ok {
            return Some(pos);
        }
        i = pos + "operator".len();
    }
    None
}

/// Take the trailing identifier run from `s`. Operates on chars so Unicode
/// identifiers (e.g. `grüßen`, `話す`) are kept whole.
fn trailing_ident(s: &str) -> &str {
    let mut start = s.len();
    for (idx, ch) in s.char_indices().rev() {
        if ch == '_' || ch == '$' || ch.is_alphanumeric() {
            start = idx;
        } else {
            break;
        }
    }
    &s[start..]
}

/// Index of the first top-level `=` that is a plain assignment (not `==`, `=>`,
/// `<=`, `+=`, etc.), or `None`.
fn find_top_eq(h: &str) -> Option<usize> {
    find_top_eq_impl(h, false)
}

/// Like [`find_top_eq`] but also skips a `=>` fat arrow. Used for Scala, where a
/// `def`'s body separator is a single `=` and `=>` is a function-type/lambda
/// arrow that must not be mistaken for the body (`def f: Int => Int`).
fn find_top_eq_scala(h: &str) -> Option<usize> {
    find_top_eq_impl(h, true)
}

fn find_top_eq_impl(h: &str, skip_arrow: bool) -> Option<usize> {
    let b = h.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            b'=' if depth == 0 => {
                let prev = if i > 0 { b[i - 1] } else { b' ' };
                let next = if i + 1 < b.len() { b[i + 1] } else { b' ' };
                let bad_prev = matches!(
                    prev,
                    b'=' | b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|'
                        | b'^' | b':'
                );
                let is_arrow = skip_arrow && next == b'>';
                if next != b'=' && !bad_prev && !is_arrow {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Given `s` starting with `(`, return the index just past the matching `)`.
fn matched_paren_end(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Given `s` starting with `<`, return the index just past the matching `>`
/// (depth-balanced), or `None`. Used to skip a C++ `template <...>` clause.
fn matched_angle_end(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
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

/// Tidy up spacing introduced when joining a multi-line signature.
fn tidy(s: &str) -> String {
    let mut t = s.to_string();
    // NOTE: no `< `/` >` collapsing here — it would mangle comparison operators
    // in expression bodies (`(a, b) => a > b` → `a> b`). Source generics are
    // already written tight (`Map<String>`), so it isn't needed.
    for (from, to) in [
        ("( ", "("),
        (" )", ")"),
        ("[ ", "["),
        (" ]", "]"),
        (" ,", ","),
        (", )", ")"),
        (",)", ")"),
        (",]", "]"),
    ] {
        t = t.replace(from, to);
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sigs(lang: Lang, src: &str) -> Vec<Signature> {
        BraceLang { lang }.extract(src)
    }

    #[test]
    fn c_line_comment_backslash_splice() {
        // C splices a `//` comment across a trailing backslash-newline, so the
        // "hidden" function on the continuation line must be suppressed.
        let src = "// comment \\\nint hidden(void) { return 0; }\nint real(void) { return 1; }\n";
        let s = sigs(Lang::C, src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "int real(void)");
        assert_eq!(s[0].line, 3);
    }

    #[test]
    fn c_multiline_define_body_skipped() {
        let src = "#define M \\\n    struct Hidden { int x; };\nint after(void) { return 0; }\n";
        let s = sigs(Lang::C, src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].kind, Kind::Constant);
        assert_eq!(s[0].text, "#define M …");
        assert_eq!(s[1].kind, Kind::Function);
        assert_eq!(s[1].text, "int after(void)");
        assert_eq!(s[1].line, 3);
    }

    #[test]
    fn cpp_constructors_recognized() {
        // Constructors have no return type; they must still be detected inside a
        // class/struct body (but a bare call elsewhere must not be).
        let src = "class Widget {\npublic:\n    Widget();\n    Widget(int v);\n    ~Widget();\n};\n";
        let s = sigs(Lang::Cpp, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class Widget", "Widget()", "Widget(int v)", "~Widget()"]);
        assert!(s[1].kind == Kind::Function);
    }

    #[test]
    fn cpp_operator_overloads_recognized() {
        let src = "struct F {\n    F operator+(const F& o) const;\n    bool operator==(const F& o) const;\n    F& operator[](int i);\n    F& operator=(const F& o);\n};\nF operator<<(int a, const F& f);\n";
        let s = sigs(Lang::Cpp, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "struct F",
                "F operator+(const F& o) const",
                "bool operator==(const F& o) const",
                "F& operator[](int i)",
                "F& operator=(const F& o)",
                "F operator<<(int a, const F& f)",
            ]
        );
    }

    #[test]
    fn cpp_raw_string_masked() {
        // Raw-string contents (fake decls + stray braces) must be ignored, and
        // the braces inside must not corrupt the nesting of later declarations.
        let src = "void before();\nconst char* s = R\"(\nvoid fake() {}\n{ {\n)\";\nvoid after();\n";
        let s = sigs(Lang::Cpp, src);
        let funcs: Vec<&str> =
            s.iter().filter(|x| x.kind == Kind::Function).map(|x| x.text.as_str()).collect();
        assert_eq!(funcs, vec!["void before()", "void after()"]);
        // `after` stays at top level — raw-string braces did not nest it.
        assert_eq!(s.last().unwrap().indent, 0);
    }

    #[test]
    fn csharp_operator_overloads_recognized() {
        let src = "public struct S {\n    public static S operator +(S a, S b) { return a; }\n    public static bool operator ==(S a, S b) { return true; }\n}\n";
        let s = sigs(Lang::CSharp, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec![
                "public struct S",
                "public static S operator +(S a, S b)",
                "public static bool operator ==(S a, S b)",
            ]
        );
    }

    #[test]
    fn csharp_where_constraint_on_own_line() {
        // The method must survive (not be dropped) and `After` must stay at the
        // class's member depth — the body brace was being orphaned before.
        let src = "public class C {\n    public T Get<T>()\n        where T : class\n    {\n        return default;\n    }\n    public void After() {}\n}\n";
        let s = sigs(Lang::CSharp, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["public class C", "public T Get<T>()", "public void After()"]);
        assert_eq!(s[1].indent, 1);
        assert_eq!(s[2].indent, 1);
    }

    #[test]
    fn rust_where_constraint_still_stripped() {
        // Regression guard: Rust multi-line `where` must keep working (clause
        // dropped, nesting intact) after the C# where-clause fix.
        let src = "pub struct Wrap<T>\nwhere\n    T: Clone,\n{\n    pub fn new(x: T) -> Self {}\n}\n";
        let s = sigs(Lang::Rust, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["pub struct Wrap<T>", "pub fn new(x: T) -> Self"]);
        assert_eq!(s[1].indent, 1);
    }

    #[test]
    fn csharp_interpolated_verbatim_masked() {
        let src = "class C {\n    void M() {\n        string s = $@\"\nclass Fake { }\n{\n\";\n    }\n    void Good() {}\n}\n";
        let s = sigs(Lang::CSharp, src);
        let funcs: Vec<&str> =
            s.iter().filter(|x| x.kind == Kind::Function).map(|x| x.text.as_str()).collect();
        assert_eq!(funcs, vec!["void M()", "void Good()"]);
        assert_eq!(s.last().unwrap().indent, 1);
    }

    #[test]
    fn dart_getters_recognized() {
        // Getters (no `(`) must be emitted; a block getter's local const must be
        // suppressed (its body is a function body, not a class body).
        let src = "class Foo {\n  int get x => _x;\n  int get y {\n    const localK = 5;\n    return 1;\n  }\n  set x(int v) { _x = v; }\n}\nint get globalValue => 42;\n";
        let s = sigs(Lang::Dart, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["class Foo", "int get x", "int get y", "set x(int v)", "int get globalValue"]
        );
    }

    #[test]
    fn dart_operators_recognized() {
        let src = "class Complex {\n  Complex operator +(Complex other) => this;\n  bool operator ==(Object o) => true;\n}\n";
        let s = sigs(Lang::Dart, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["class Complex", "Complex operator +(Complex other)", "bool operator ==(Object o)"]
        );
    }

    #[test]
    fn dart_class_modifiers_recognized() {
        // base/interface/final class are Dart 3 modifiers; `final answer = …`
        // must still be a constant (final is dual-use).
        let src = "base class A {\n  void s() {}\n}\ninterface class B {}\nfinal class C {}\nfinal answer = 42;\n";
        let s = sigs(Lang::Dart, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["base class A", "void s()", "interface class B", "final class C", "final answer = …"]
        );
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s.last().unwrap().kind, Kind::Constant);
    }

    #[test]
    fn go_type_group_block() {
        let src = "package main\n\ntype (\n\tID int64\n\tName string\n\tHandler func(ID) error\n)\n";
        let s = sigs(Lang::Go, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["type ID int64", "type Name string", "type Handler func(ID) error"]);
        assert!(s.iter().all(|x| x.kind == Kind::Class));
    }

    #[test]
    fn go_raw_string_trailing_backslash() {
        // A Go raw string is not escape-processed; a trailing `\` must not eat the
        // closing backtick and swallow following declarations.
        let src = "package main\n\nvar s = `x\\`\n\nfunc Visible() {}\n";
        let s = sigs(Lang::Go, src);
        let funcs: Vec<&str> =
            s.iter().filter(|x| x.kind == Kind::Function).map(|x| x.text.as_str()).collect();
        assert_eq!(funcs, vec!["func Visible()"]);
    }

    #[test]
    fn java_keyword_like_method_names() {
        // Methods named after control keywords (`of`, `with`, `in`) must be kept
        // when they have a return type; a bare `if (...)` is still rejected.
        let src = "class F {\n  static F of(String n) { return null; }\n  F with(String k) { return this; }\n  void run() {\n    if (x) { return; }\n  }\n}\n";
        let s = sigs(Lang::Java, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class F", "static F of(String n)", "F with(String k)", "void run()"]);
    }

    #[test]
    fn java_annotation_default_array() {
        let src = "@interface Ann {\n  String[] tags() default {};\n  String[] more() default {\"a\", \"b\"};\n}\n";
        let s = sigs(Lang::Java, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["@interface Ann", "String[] tags() default {}", "String[] more() default {\"a\", \"b\"}"]
        );
    }

    #[test]
    fn js_arrow_comparison_operators_not_mangled() {
        let src = "const f = (a, b) => a > b;\nconst g = (a, b) => a < b;\n";
        let s = sigs(Lang::Js, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["const f = (a, b) => a > b", "const g = (a, b) => a < b"]);
    }

    #[test]
    fn js_computed_method_names() {
        let src = "class Foo {\n  normalMethod() {}\n  [Symbol.iterator]() {}\n  static [Symbol.hasInstance](obj) {}\n}\n";
        let s = sigs(Lang::Js, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["class Foo", "normalMethod()", "[Symbol.iterator]()", "static [Symbol.hasInstance](obj)"]
        );
    }

    #[test]
    fn scala_lower_bound_not_mangled() {
        // Regression guard for the `tidy` change: Scala `[B >: A]` lower bound.
        let src = "trait T {\n  def getOrElse[B >: A](default: B): B\n}\n";
        let s = sigs(Lang::Scala, src);
        assert!(s.iter().any(|x| x.text == "def getOrElse[B >: A](default: B): B"));
    }

    #[test]
    fn kotlin_typealias() {
        let src = "typealias StringList = List<String>\ntypealias T<A, B> = (A) -> B\n";
        let s = sigs(Lang::Kotlin, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["typealias StringList = List<String>", "typealias T<A, B> = (A) -> B"]);
        assert!(s.iter().all(|x| x.kind == Kind::Class));
    }

    #[test]
    fn kotlin_init_block_suppresses_locals() {
        let src = "class Foo {\n    init {\n        val x: Int = 42\n        var y: String = \"hi\"\n    }\n    fun bar() {}\n}\n";
        let s = sigs(Lang::Kotlin, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class Foo", "fun bar()"]);
    }

    #[test]
    fn kotlin_property_without_initializer() {
        let src = "class Foo {\n    val fullName: String\n        get() = \"x\"\n    val n: Int = 5\n}\n";
        let s = sigs(Lang::Kotlin, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class Foo", "val fullName: String", "val n: Int = …"]);
    }

    #[test]
    fn where_clause_brace_on_same_line() {
        // Regression guard: a `where` clause with `{` on its own line must not
        // run to EOF and swallow following declarations.
        let src = "class A<T>\n    where T : Any {\n    fun foo(): T? = null\n}\nclass B {\n    fun bar(): Int = 1\n}\n";
        let s = sigs(Lang::Kotlin, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class A<T>", "fun foo(): T?", "class B", "fun bar(): Int"]);
    }

    #[test]
    fn php_heredoc_body_ignored() {
        let src = "<?php\nfunction f(): string {\n    return <<<EOT\n    function fake() {}\n    EOT;\n}\nfunction g(): void {}\n";
        let s = sigs(Lang::Php, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["function f(): string", "function g(): void"]);
    }

    #[test]
    fn php_nowdoc_braces_dont_corrupt_nesting() {
        let src = "<?php\nclass C {\n    public function m(): string {\n        return <<<'EOT'\n        }}}\n        EOT;\n    }\n    public function after(): int { return 1; }\n}\n";
        let s = sigs(Lang::Php, src);
        // `after` must remain a member of C (indent 1), not hoisted to top level.
        let after = s.iter().find(|x| x.text.contains("after")).unwrap();
        assert_eq!(after.indent, 1);
    }

    #[test]
    fn php_inline_attribute_in_params() {
        let src = "<?php\nfunction f(#[Inject] string $x): void {}\nfunction g(int $y): int { return $y; }\n";
        let s = sigs(Lang::Php, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["function f(#[Inject] string $x): void", "function g(int $y): int"]);
    }

    #[test]
    fn scala_enum_and_function_type_return() {
        let src = "enum Color {\n  case Red\n  def isWarm: Boolean = true\n}\ntrait T {\n  def adder: Int => Int\n}\n";
        let s = sigs(Lang::Scala, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["enum Color", "def isWarm: Boolean", "trait T", "def adder: Int => Int"]);
        assert_eq!(s[0].kind, Kind::Class);
    }

    #[test]
    fn scala_operator_method_does_not_eat_siblings() {
        // Regression guard: a `def <` operator name must not push the angle
        // counter and swallow the rest of the file.
        let src = "class Ord {\n  def <(o: Ord): Boolean = true\n  def kept(): String = \"x\"\n}\ndef topLevel(): Int = 1\n";
        let s = sigs(Lang::Scala, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class Ord", "def <(o: Ord): Boolean", "def kept(): String", "def topLevel(): Int"]);
    }

    #[test]
    fn scala_access_qualifier() {
        let src = "class X {\n  private[this] val a: Int = 1\n  protected[pkg] def b(): Int = 2\n}\n";
        let s = sigs(Lang::Scala, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["class X", "private[this] val a: Int = …", "protected[pkg] def b(): Int"]);
    }

    #[test]
    fn swift_prefix_postfix_nonisolated_modifiers() {
        let src = "struct S {\n  prefix static func - (v: S) -> S { v }\n}\npostfix func +++ (n: Int) -> Int { n }\nactor A {\n  nonisolated func c() -> Int { 0 }\n}\n";
        let s = sigs(Lang::Swift, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["struct S", "prefix static func - (v: S) -> S", "postfix func +++ (n: Int) -> Int", "actor A", "nonisolated func c() -> Int"]
        );
    }

    #[test]
    fn ts_declare_statements() {
        let src = "declare class ET {\n  on(t: string): void;\n}\ndeclare const w: Window;\ndeclare enum D { Up }\n";
        let s = sigs(Lang::Ts, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(texts, vec!["declare class ET", "on(t: string): void", "declare const w: Window = …", "declare enum D"]);
    }

    #[test]
    fn ts_optional_members_and_intersection() {
        let src = "interface P {\n  required(): void;\n  optional?(): void;\n}\ntype A = { a: string } & { b: number };\n";
        let s = sigs(Lang::Ts, src);
        let texts: Vec<&str> = s.iter().map(|x| x.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["interface P", "required(): void", "optional?(): void", "type A = { a: string } & { b: number }"]
        );
    }

    #[test]
    fn java_line_comment_no_splice() {
        // Java does NOT perform backslash-newline splicing: the comment ends at
        // the newline and the next line is real code.
        let src = "// comment \\\nclass Real {}\n";
        let s = sigs(Lang::Java, src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[0].text, "class Real");
    }

    #[test]
    fn rust_top_level_functions() {
        let src = "pub fn foo(a: i32, b: i32) -> i32 {\n    a + b\n}\nfn bar() {}\n";
        let s = sigs(Lang::Rust, src);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].kind, Kind::Function);
        assert_eq!(s[0].text, "pub fn foo(a: i32, b: i32) -> i32");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[0].line, 1);
        assert_eq!(s[1].text, "fn bar()");
        assert_eq!(s[1].line, 4);
    }

    #[test]
    fn rust_type_with_nested_members() {
        let src = "struct Point {\n    x: i32,\n}\nimpl Point {\n    fn new() -> Self {}\n    pub fn x(&self) -> i32 {}\n}\n";
        let s = sigs(Lang::Rust, src);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[0].text, "struct Point");
        assert_eq!(s[0].indent, 0);
        // Methods nest under the `impl` block.
        assert_eq!(s[1].kind, Kind::Function);
        assert_eq!(s[1].text, "fn new() -> Self");
        assert_eq!(s[1].indent, 1);
        assert_eq!(s[1].line, 5);
        assert_eq!(s[2].text, "pub fn x(&self) -> i32");
        assert_eq!(s[2].indent, 1);
    }

    #[test]
    fn rust_multiline_signature_joined() {
        let src = "fn long(\n    a: i32,\n    b: i32,\n) -> Result<i32, Error> {\n}\n";
        let s = sigs(Lang::Rust, src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "fn long(a: i32, b: i32) -> Result<i32, Error>");
    }

    #[test]
    fn rust_constants_detected_and_rejected() {
        let src = "pub const MAX_SIZE: usize = 1024;\nstatic GREETING: &str = \"hi\";\nlet local_var = compute();\n";
        let consts: Vec<String> = sigs(Lang::Rust, src)
            .into_iter()
            .filter(|x| x.kind == Kind::Constant)
            .map(|x| x.text)
            .collect();
        assert_eq!(
            consts,
            vec![
                "pub const MAX_SIZE: usize = …".to_string(),
                "static GREETING: &str = …".to_string(),
            ]
        );
        // `let local_var = ...` is not a module constant and must be ignored.
        assert!(sigs(Lang::Rust, src).iter().all(|s| !s.text.contains("local_var")));
    }

    #[test]
    fn rust_decl_in_comment_and_string_ignored() {
        let src = "// fn commented_out() {}\nfn real() {\n    let s = \"fn fake() {}\";\n}\n";
        let s = sigs(Lang::Rust, src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "fn real()");
        assert_eq!(s[0].line, 2);
    }

    #[test]
    fn rust_lifetime_not_treated_as_char() {
        let src = "fn borrow<'a>(x: &'a str) -> &'a str {}\n";
        let s = sigs(Lang::Rust, src);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "fn borrow<'a>(x: &'a str) -> &'a str");
    }

    #[test]
    fn go_declarations() {
        let src = "package main\n\nconst Version = \"1.0\"\n\ntype Server struct {\n}\n\nfunc (s *Server) Start(addr string) error {\n}\n\nfunc main() {\n}\n";
        let s = sigs(Lang::Go, src);
        let texts: Vec<(&str, usize)> = s.iter().map(|x| (x.text.as_str(), x.indent)).collect();
        assert_eq!(
            texts,
            vec![
                ("const Version = …", 0),
                ("type Server struct", 0),
                ("func (s *Server) Start(addr string) error", 0),
                ("func main()", 0),
            ]
        );
    }

    #[test]
    fn js_class_methods_and_const() {
        let src = "class Animal {\n  constructor(name) {\n    this.name = name;\n  }\n  speak() {\n    return this.name;\n  }\n}\nconst SOUND = \"woof\";\n";
        let s = sigs(Lang::Js, src);
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[0].text, "class Animal");
        assert_eq!(s[0].indent, 0);
        assert_eq!(s[1].text, "constructor(name)");
        assert_eq!(s[1].indent, 1);
        assert_eq!(s[2].text, "speak()");
        assert_eq!(s[2].indent, 1);
        let last = s.last().unwrap();
        assert_eq!(last.kind, Kind::Constant);
        assert_eq!(last.text, "const SOUND = …");
    }

    #[test]
    fn ts_class_type_and_const() {
        let src = "export class Circle {\n    radius: number;\n    area(): number {\n        return 3;\n    }\n}\ntype ID = string;\nconst PI: number = 3.14;\n";
        let s = sigs(Lang::Ts, src);
        assert_eq!(s[0].text, "export class Circle");
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[1].text, "area(): number");
        assert_eq!(s[1].kind, Kind::Function);
        assert_eq!(s[1].indent, 1);
        assert!(s.iter().any(|x| x.text == "type ID = string" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "const PI: number = …" && x.kind == Kind::Constant));
        // The field `radius: number;` is not a function or constant.
        assert!(s.iter().all(|x| !x.text.contains("radius")));
    }

    #[test]
    fn java_class_method_and_constant() {
        let src = "public class Calc {\n    public static final int MAX = 100;\n    private int value;\n    public int add(int a, int b) {\n        return a + b;\n    }\n}\n";
        let s = sigs(Lang::Java, src);
        assert_eq!(s[0].text, "public class Calc");
        assert_eq!(s[0].kind, Kind::Class);
        assert_eq!(s[1].text, "public static final int MAX = …");
        assert_eq!(s[1].kind, Kind::Constant);
        assert_eq!(s[1].indent, 1);
        assert!(s.iter().any(|x| x.text == "public int add(int a, int b)"
            && x.kind == Kind::Function
            && x.indent == 1));
        // The non-final field `value` is not a constant.
        assert!(s.iter().all(|x| !x.text.contains("value")));
    }

    #[test]
    fn c_define_function_struct_const() {
        let src = "#define MAX 100\nint add(int a, int b) {\n    return a + b;\n}\nstruct Point {\n    int x;\n    int y;\n};\nstatic const double PI = 3.14;\n";
        let s = sigs(Lang::C, src);
        assert_eq!(s[0].text, "#define MAX …");
        assert_eq!(s[0].kind, Kind::Constant);
        assert_eq!(s[1].text, "int add(int a, int b)");
        assert_eq!(s[1].kind, Kind::Function);
        assert_eq!(s[1].indent, 0);
        assert!(s.iter().any(|x| x.text == "struct Point" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "static const double PI = …" && x.kind == Kind::Constant));
    }

    #[test]
    fn c_prototype_detected_call_rejected() {
        let src = "int compute(int n);\nvoid run(void) {\n    compute(5);\n}\n";
        let s = sigs(Lang::C, src);
        assert_eq!(s[0].text, "int compute(int n)");
        assert_eq!(s[1].text, "void run(void)");
        // The call `compute(5);` inside the body is not a declaration.
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn never_panics_on_weird_input() {
        let weird = [
            "fn (((((",
            "/* unterminated",
            "\"unterminated string",
            "'\\",
            "r#\"raw without end",
            "}}}}}}",
            "\u{1F600} fn x() {}",
            "",
        ];
        for lang in [
            Lang::Rust,
            Lang::Go,
            Lang::Js,
            Lang::Ts,
            Lang::Java,
            Lang::C,
            Lang::Cpp,
            Lang::CSharp,
            Lang::Kotlin,
            Lang::Swift,
            Lang::Php,
            Lang::Scala,
            Lang::Dart,
        ] {
            for w in weird {
                let _ = BraceLang { lang }.extract(w);
            }
        }
    }

    #[test]
    fn cpp_class_template_func_namespace_const() {
        let src = "namespace geo {\nclass Point {\npublic:\n    int x() const { return x_; }\n};\n}\ntemplate <typename T>\nT max_of(T a, T b) {\n    return a > b ? a : b;\n}\nconstexpr double PI = 3.14159;\n// int commented(int z) {}\n";
        let s = sigs(Lang::Cpp, src);
        assert!(s.iter().any(|x| x.text == "namespace geo" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "class Point" && x.kind == Kind::Class));
        assert!(s
            .iter()
            .any(|x| x.text == "int x() const" && x.kind == Kind::Function && x.indent == 2));
        // The `template <...>` clause sits on its own line; the function header
        // is still recognized on the following line.
        assert!(s
            .iter()
            .any(|x| x.text == "T max_of(T a, T b)" && x.kind == Kind::Function));
        assert!(s
            .iter()
            .any(|x| x.text == "constexpr double PI = …" && x.kind == Kind::Constant));
        // Declaration inside a comment must be ignored.
        assert!(s.iter().all(|x| !x.text.contains("commented")));
    }

    #[test]
    fn csharp_class_method_const_namespace() {
        let src = "namespace App {\n    public class Calc {\n        public const int Max = 100;\n        public int Add(int a, int b) {\n            return a + b;\n        }\n    }\n    interface IShape {\n    }\n}\n/* class Hidden {} */\n";
        let s = sigs(Lang::CSharp, src);
        assert!(s.iter().any(|x| x.text == "namespace App" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "public class Calc" && x.kind == Kind::Class));
        assert!(s
            .iter()
            .any(|x| x.text == "public const int Max = …" && x.kind == Kind::Constant));
        assert!(s.iter().any(
            |x| x.text == "public int Add(int a, int b)" && x.kind == Kind::Function && x.indent == 2
        ));
        // `static readonly` is recognized as a constant too.
        assert!(sigs(Lang::CSharp, "static readonly int N = 5;\n")
            .iter()
            .any(|x| x.text == "static readonly int N = …" && x.kind == Kind::Constant));
        assert!(s.iter().any(|x| x.text == "interface IShape" && x.kind == Kind::Class));
        assert!(s.iter().all(|x| !x.text.contains("Hidden")));
    }

    #[test]
    fn kotlin_class_fun_val() {
        let src = "data class User(val id: Int) {\n    fun greet(name: String): String {\n        return name\n    }\n}\nconst val MAX = 100\nobject Registry {\n    val items = listOf<String>()\n}\n// fun hidden() {}\n";
        let s = sigs(Lang::Kotlin, src);
        assert!(s.iter().any(|x| x.text == "data class User(val id: Int)" && x.kind == Kind::Class));
        assert!(s.iter().any(
            |x| x.text == "fun greet(name: String): String" && x.kind == Kind::Function && x.indent == 1
        ));
        assert!(s.iter().any(|x| x.text == "const val MAX = …" && x.kind == Kind::Constant));
        assert!(s.iter().any(|x| x.text == "object Registry" && x.kind == Kind::Class));
        assert!(s.iter().all(|x| !x.text.contains("hidden")));
    }

    #[test]
    fn swift_struct_func_let_protocol() {
        let src = "struct Circle {\n    let radius: Double\n    func area() -> Double {\n        return 3.14 * radius * radius\n    }\n}\nprotocol Shape {\n    func draw()\n}\nlet PI = 3.14159\n// func hidden() {}\n";
        let s = sigs(Lang::Swift, src);
        assert!(s.iter().any(|x| x.text == "struct Circle" && x.kind == Kind::Class));
        assert!(s.iter().any(
            |x| x.text == "func area() -> Double" && x.kind == Kind::Function && x.indent == 1
        ));
        assert!(s.iter().any(|x| x.text == "protocol Shape" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "let PI = …" && x.kind == Kind::Constant));
        assert!(s.iter().all(|x| !x.text.contains("hidden")));
    }

    #[test]
    fn php_class_function_const() {
        let src = "<?php\nclass Calc {\n    const MAX = 100;\n    public function add($a, $b) {\n        return $a + $b;\n    }\n}\nfunction helper($x) {\n    return $x;\n}\n# function hidden() {}\n// const HIDDEN = 1;\n";
        let s = sigs(Lang::Php, src);
        assert!(s.iter().any(|x| x.text == "class Calc" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "const MAX = …" && x.kind == Kind::Constant));
        assert!(s.iter().any(
            |x| x.text == "public function add($a, $b)" && x.kind == Kind::Function && x.indent == 1
        ));
        assert!(s.iter().any(|x| x.text == "function helper($x)" && x.kind == Kind::Function));
        assert!(s.iter().all(|x| !x.text.contains("hidden")));
        assert!(s.iter().all(|x| !x.text.contains("HIDDEN")));
    }

    #[test]
    fn scala_class_def_val_caseclass() {
        let src = "class Animal(name: String) {\n  def speak(): String = {\n    name\n  }\n}\ncase class Point(x: Int, y: Int)\ntrait Shape {\n  def area: Double\n}\nval PI = 3.14159\n/* def hidden() = 1 */\n";
        let s = sigs(Lang::Scala, src);
        assert!(s.iter().any(|x| x.text == "class Animal(name: String)" && x.kind == Kind::Class));
        assert!(s.iter().any(
            |x| x.text == "def speak(): String" && x.kind == Kind::Function && x.indent == 1
        ));
        assert!(s.iter().any(|x| x.text == "case class Point(x: Int, y: Int)" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "trait Shape" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "val PI = …" && x.kind == Kind::Constant));
        assert!(s.iter().all(|x| !x.text.contains("hidden")));
    }

    #[test]
    fn dart_class_method_const_mixin() {
        let src = "class Calc {\n  int add(int a, int b) {\n    return a + b;\n  }\n}\nmixin Logger {\n  void log(String m) {\n    print(m);\n  }\n}\nconst pi = 3.14159;\nvoid main() {\n  print('hi');\n}\n// void hidden() {}\n";
        let s = sigs(Lang::Dart, src);
        assert!(s.iter().any(|x| x.text == "class Calc" && x.kind == Kind::Class));
        assert!(s.iter().any(
            |x| x.text == "int add(int a, int b)" && x.kind == Kind::Function && x.indent == 1
        ));
        assert!(s.iter().any(|x| x.text == "mixin Logger" && x.kind == Kind::Class));
        assert!(s.iter().any(|x| x.text == "const pi = …" && x.kind == Kind::Constant));
        assert!(s.iter().any(|x| x.text == "void main()" && x.kind == Kind::Function));
        assert!(s.iter().all(|x| !x.text.contains("hidden")));
    }

    // ---- regression tests for issues surfaced by the test fixtures ----

    #[test]
    fn ts_generic_method_recognized() {
        // A type parameter between the name and `(` must not hide the method.
        let src = "class C {\n  identity<T>(x: T): T { return x; }\n  plain(a: number): number { return a; }\n}\n";
        let s = sigs(Lang::Ts, src);
        assert!(
            s.iter().any(|x| x.text == "identity<T>(x: T): T" && x.kind == Kind::Function),
            "got: {s:?}"
        );
        assert!(s.iter().any(|x| x.text == "plain(a: number): number"));
    }

    #[test]
    fn unicode_constant_names_recognized() {
        // `take_ident` must read non-ASCII identifiers (was ASCII-only).
        let kt = sigs(Lang::Kotlin, "const val ÜBER_LIMIT = 5\nconst val MAX = 10\n");
        assert!(kt.iter().any(|x| x.text == "const val ÜBER_LIMIT = …" && x.kind == Kind::Constant));
        let go = sigs(Lang::Go, "const Δ = 1\nconst 最大値 = 9\n");
        assert!(go.iter().any(|x| x.text == "const Δ = …"));
        assert!(go.iter().any(|x| x.text == "const 最大値 = …"));
    }

    #[test]
    fn expression_bodies_dropped() {
        // Kotlin `= expr`, C#/Dart `=> expr` bodies must not leak into the text.
        let kt = sigs(Lang::Kotlin, "class A {\n  fun greet(): String = \"hi\"\n}\n");
        assert!(kt.iter().any(|x| x.text == "fun greet(): String" && x.kind == Kind::Function));
        let cs = sigs(Lang::CSharp, "class A {\n  double balance() => 0;\n}\n");
        assert!(cs.iter().any(|x| x.text == "double balance()" && x.kind == Kind::Function));
        let dart = sigs(Lang::Dart, "int add(int a, int b) => a + b;\n");
        assert!(dart.iter().any(|x| x.text == "int add(int a, int b)" && x.kind == Kind::Function));
        // A default parameter `=` must NOT be mistaken for an expression body.
        let kt2 = sigs(Lang::Kotlin, "fun f(x: Int = 5) {}\n");
        assert!(kt2.iter().any(|x| x.text == "fun f(x: Int = 5)"), "got: {kt2:?}");
    }
}
