//! This file tests that the extractor ignores `fn`/`struct`/`const`/`impl`
//! declarations hidden inside various comment and string forms.
//!
//! The doc-comment below is NOT a real declaration:
//! ```
//! fn doc_example_not_real() {}
//! struct DocStruct { x: i32 }
//! ```

// fn line_comment_fn() {} -- must be ignored
// struct LineStruct { x: i32 } -- must be ignored
// const LINE_CONST: u32 = 0; -- must be ignored

/* fn block_comment_fn() { } */
/* struct BlockStruct; */
/*
 * Multi-line block comment.
 * const BLOCK_CONST: u32 = 99;
 * impl SomeType { fn method() {} }
 */

/** fn javadoc_style_fn() {} -- inside block comment, must be ignored */

/// Real constant below; the previous comments must not be extracted.
pub const REAL_CONST: u32 = 42;

/// Real struct.
pub struct Config {
    /// `fn hidden_in_field_doc()` — doc comment on a field, must be ignored.
    pub value: u32,
}

impl Config {
    /// Creates a new config.
    /// Example usage:
    /// ```rust
    /// fn example_in_doc() {}
    /// const EXAMPLE_CONST: u32 = 1;
    /// ```
    pub fn new(value: u32) -> Self {
        Config { value }
    }

    pub fn describe(&self) -> &'static str {
        // fn not_a_decl_in_body() {} -- comment inside a function body
        "this is a normal string"
    }

    pub fn raw_describe(&self) -> &'static str {
        r"raw string with fn fake_raw_fn() and struct FakeStruct inside"
    }

    pub fn hashed_raw(&self) -> &'static str {
        r#"fn still_fake() { struct AlsoFake; const ALSO_FAKE: u32 = 0; }"#
    }

    pub fn multi_hashed(&self) -> &'static str {
        r##"struct Super##Fake { fn nested_fake() {} }"##
    }

    pub fn string_with_escapes(&self) -> String {
        let s = "fn escaped_not_real() \"quoted body\" {}";
        s.to_owned()
    }
}

/// A real trait — the fakes above must not appear in output.
pub trait Describable {
    fn describe(&self) -> &'static str;
}

impl Describable for Config {
    fn describe(&self) -> &'static str {
        // The string literal on the next line contains fake keywords.
        let _msg = "impl FakeImpl { fn fake_method(&self) {} }";
        "config"
    }
}

/// Real free function.
pub fn process(cfg: &Config) -> u32 {
    /* fn hidden_in_block() {} */
    cfg.value * 2
}

static HELP_TEXT: &str = "
Usage: myapp [OPTIONS]

Options:
  --fn-flag    fn not_real() because inside a string
  --struct-out struct NotReal { value: u32 }
";
