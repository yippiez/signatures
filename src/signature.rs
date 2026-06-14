//! The language-agnostic unit that every extractor produces.

/// What kind of declaration a signature represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Function,
    Class,
    Constant,
}

/// A single extracted signature: a declaration line with its body removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// Nesting level (0 = top level). Used purely for indentation on output.
    pub indent: usize,
    pub kind: Kind,
    /// The normalized declaration text, e.g. `def foo(a, b):` or `MAX = …`.
    pub text: String,
    /// 1-based source line where the declaration starts.
    pub line: usize,
    /// When this signature elided a value/RHS, the full (collapsed) declaration
    /// text WITHOUT the `…` elision, e.g. `MAX = 1 << 20`. `None` when nothing
    /// was elided; renderers fall back to `text` in that case.
    pub full: Option<String>,
}
