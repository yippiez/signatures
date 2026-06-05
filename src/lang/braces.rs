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
        let mlines: Vec<&str> = masked.lines().collect();
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
                i += 1;
                continue;
            }

            if !prefilter(trimmed, self.lang) {
                update_stack(&mut stack, &mlines[i..=i], None);
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

            let mut pending: Option<bool> = None;
            if let Some((kind, text)) = classify(&header, term, self.lang) {
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

/// Handle a Go grouped `const`/`var` block: `const ( A = 1\n B = 2 )`. Emits one
/// constant per member. Returns the number of lines consumed, or `None` if the
/// line at `start` is not such a block.
fn go_group_block(
    mlines: &[&str],
    start: usize,
    out: &mut Vec<Signature>,
    indent: usize,
) -> Option<usize> {
    let first = mlines[start].trim_start();
    let (kw, after) = take_ident(first);
    if !matches!(kw, "const" | "var") {
        return None;
    }
    if !after.trim_start().starts_with('(') {
        return None;
    }

    let mut depth: i32 = 0;
    let mut consumed = 0usize;
    let mut k = start;
    while k < mlines.len() {
        consumed += 1;
        let raw = mlines[k];
        // At group level (already inside the parens), the leading identifier of
        // a line names a constant.
        if depth >= 1 {
            let t = raw.trim_start();
            if !t.starts_with(')') {
                let (name, _) = take_ident(t);
                if !name.is_empty() && !is_control(name) {
                    out.push(Signature {
                        indent,
                        kind: Kind::Constant,
                        text: format!("{kw} {name} = …"),
                        line: k + 1,
                    });
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
        // masked stream stays index-aligned with the original source).
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        // PHP line comment `# ...` (also covers `#[Attr]` attribute lines).
        if lang == Lang::Php && c == '#' {
            while i < n && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
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
                i = mask_quoted(&chars, i, '"', false, &mut out);
                continue;
            }
            '`' if matches!(lang, Lang::Go | Lang::Js | Lang::Ts) => {
                // Go raw string / JS template literal: spans newlines.
                let escape = matches!(lang, Lang::Js | Lang::Ts);
                i = mask_quoted(&chars, i, '`', true, &mut out);
                let _ = escape; // escapes handled inside mask_quoted for backtick
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
                        i = mask_quoted(&chars, i, '\'', false, &mut out);
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
fn mask_quoted(chars: &[char], start: usize, delim: char, allow_newline: bool, out: &mut String) -> usize {
    let n = chars.len();
    out.push(' ');
    let mut i = start + 1;
    // Backtick (JS template) and normal quotes honor backslash escapes; Go raw
    // strings (also backtick) do not, but treating an escaped backtick as
    // escaped is harmless for masking purposes.
    while i < n {
        let c = chars[i];
        if c == '\\' && delim != '`' && i + 1 < n {
            out.push(' ');
            out.push(' ');
            i += 2;
            continue;
        }
        if c == '\\' && delim == '`' && i + 1 < n {
            out.push(' ');
            out.push(' ');
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
    let n = chars.len();
    out.push(' '); // @
    out.push(' '); // opening "
    let mut i = start + 2;
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
                '<' if paren <= 0 => {
                    angle += 1;
                    piece.push(och(j));
                    j += 1;
                }
                '>' if paren <= 0 => {
                    if angle > 0 {
                        angle -= 1;
                    }
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
    let mut p = j;
    while p > lead {
        p -= 1;
        if !mchars[p].is_whitespace() {
            if matches!(mchars[p], '=' | ':' | '|' | ',') {
                return true;
            }
            break;
        }
    }
    if j > lead && !mchars[j - 1].is_whitespace() {
        let mut s = j;
        while s > lead && (mchars[s - 1].is_alphanumeric() || mchars[s - 1] == '_') {
            s -= 1;
        }
        let word: String = mchars[s..j].iter().collect();
        if (word == "interface" || word == "struct")
            && (s == lead || !(mchars[s - 1].is_alphanumeric() || mchars[s - 1] == '_'))
        {
            return true;
        }
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
            let name = trailing_ident(t[..idx].trim_end());
            if !name.is_empty() && !is_control(name) {
                return true;
            }
        }
    }

    false
}

/// Decide what kind of declaration a gathered header is, returning the kind and
/// its normalized text (body already removed by `gather`).
fn classify(header: &str, term: Term, lang: Lang) -> Option<(Kind, String)> {
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
        if h.contains('(') && looks_like_function(h, lang, term) {
            return Some((Kind::Function, h.to_string()));
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
                let r = rest.trim_start();
                // `static mut X` — skip an inner `mut`.
                let r = r.strip_prefix("mut").map(|s| s.trim_start()).unwrap_or(r);
                let (n2, _) = take_ident(r);
                if n2.is_empty() {
                    return None; // bare `const` / `static` with no name
                }
                if n2 == "fn" {
                    return Some((Kind::Function, h.to_string()));
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
                if looks_like_function(h, lang, term) {
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
                return Some((Kind::Function, h.to_string()));
            }
            if matches!(kw, "val" | "var") {
                let (n2, _) = take_ident(rest.trim_start());
                if n2.is_empty() {
                    return None;
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
    ) && looks_like_function(h, lang, term)
    {
        return Some((Kind::Function, h.to_string()));
    }

    None
}

/// Scala `def` text: drop a trailing `= body` / `= {` so only the header shows.
fn scala_def_text(h: &str) -> String {
    if let Some(eq) = find_top_eq(h) {
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
fn looks_like_function(h: &str, lang: Lang, term: Term) -> bool {
    let idx = match h.find('(') {
        Some(i) => i,
        None => return false,
    };
    let before = h[..idx].trim_end();
    let name = trailing_ident(before);
    if name.is_empty() || is_control(name) {
        return false;
    }
    let prefix = before[..before.len() - name.len()].trim();
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
        Lang::C | Lang::Cpp | Lang::CSharp | Lang::Java | Lang::Dart => {
            !prefix.is_empty() && (term == Term::Brace || term == Term::Semi)
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
        Lang::Kotlin => &["class", "interface", "object"],
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
        Lang::Scala => &["class", "trait", "object", "type"],
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
        ),
        Lang::Scala => matches!(
            w,
            "case" | "sealed" | "implicit" | "lazy" | "final"
        ),
        Lang::Php => matches!(w, "final" | "readonly"),
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
/// rest of the string.
fn take_ident(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'_' || c == b'$' || c.is_ascii_alphanumeric() {
            i += 1;
        } else {
            break;
        }
    }
    (&s[..i], &s[i..])
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
                if next != b'=' && !bad_prev {
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
    for (from, to) in [
        ("( ", "("),
        (" )", ")"),
        ("[ ", "["),
        (" ]", "]"),
        (" ,", ","),
        (", )", ")"),
        (",)", ")"),
        (",]", "]"),
        ("< ", "<"),
        (" >", ">"),
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
}
