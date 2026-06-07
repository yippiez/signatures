@@CASE@@ comments_strings
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
@@CASE@@ const_continuation_multiline_fn
const A: &str = "hello \
    world";

fn foo(
    x: i32,
) -> i32 { x }
@@CASE@@ const_line_continuation
const A: &str = "hello \
    world";
const B: u32 = 100;
@@CASE@@ const_lt
const A: bool = 1 < 4;
const B: u32 = 100;
@@CASE@@ const_shl
const A: u32 = 1 << 4;
const B: u32 = 100;
@@CASE@@ const_unsafe_fn
const unsafe fn my_fn(x: i32) -> i32 { x }
@@CASE@@ edge
//! Edge cases and malformed-but-parseable constructs.
//! The extractor must not panic on any of these inputs.

// ── 1. Empty / whitespace-only bodies ───────────────────────────────────────

pub struct Empty {}

pub enum Void {}

pub trait Marker {}

// ── 2. const fn and static mut ──────────────────────────────────────────────

pub const fn square(n: u32) -> u32 {
    n * n
}

pub static mut COUNTER: u64 = 0;

// ── 3. One-liner signatures ──────────────────────────────────────────────────

pub fn noop() {}
pub fn identity<T>(x: T) -> T { x }
pub fn add(a: i32, b: i32) -> i32 { a + b }

// ── 4. Type aliases ──────────────────────────────────────────────────────────

pub type Result<T> = std::result::Result<T, String>;
pub type Callback = Box<dyn Fn(u32) -> u32 + Send + Sync>;

// ── 5. Union (unsafe feature) ────────────────────────────────────────────────

pub union Bits {
    pub as_u32: u32,
    pub as_f32: f32,
}

// ── 6. Impl with no methods (just a marker) ──────────────────────────────────

pub struct Opaque(u64);

impl Opaque {}

// ── 7. Attribute macros on items ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

// ── 8. Semicolon-terminated signatures (trait method declarations) ───────────

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
    fn serialized_size(&self) -> usize;
}

pub trait Deserialize: Sized {
    fn deserialize(bytes: &[u8]) -> Result<Self>;
    fn expected_size() -> usize;
}

// ── 9. Const with complex type expression ───────────────────────────────────

pub const LOOKUP: [u8; 4] = [0, 1, 2, 3];
pub const MASK: u64 = 0xDEAD_BEEF_CAFE_0000;

// ── 10. Extern block (FFI-style; body content ignored) ──────────────────────

extern "C" {
    fn c_sqrt(x: f64) -> f64;
    fn c_abs(x: i32) -> i32;
}

// ── 11. Unclosed / dangling items (malformed but parseable) ─────────────────

// The following function has its closing brace on the same line as the last
// statement, with unusual spacing — still valid Rust.
pub fn compact(x:u32)->u32{x+1}

// An item whose where-clause trails past end-of-typical-line budget:
pub struct Constrained<A, B, C>
where
    A: Clone,
    B: Clone,
    C: Clone,
{
    a: A,
    b: B,
    c: C,
}

impl<A, B, C> Constrained<A, B, C>
where
    A: Clone + std::fmt::Debug,
    B: Clone + std::fmt::Debug,
    C: Clone + std::fmt::Debug,
{
    pub fn new(a: A, b: B, c: C) -> Self {
        Constrained { a, b, c }
    }
}
@@CASE@@ extern_abi_decl
extern "C" fn c_abi() -> i32;
pub unsafe extern "C" fn full_c_abi(x: *mut u8) -> bool;
extern "Rust" fn rust_abi() {};
extern "system" fn sys_abi() {};
@@CASE@@ extern_abi_fn
extern "C" fn c_style() -> i32 { 1 }
pub extern "C" fn pub_c_style() -> i32 { 2 }
extern "Rust" fn rust_abi() -> i32 { 3 }
extern fn default_abi() -> i32 { 4 }
fn normal() -> i32 { 5 }
@@CASE@@ extern_block
extern "C" {
    fn ext_fn_one() -> i32;
    fn ext_fn_two(x: f64) -> bool;
}

fn after() {}
@@CASE@@ generics
//! Multi-line generic signatures, where-clauses, associated types, and
//! higher-ranked trait bounds.  All headers span multiple source lines.

use std::fmt;
use std::marker::PhantomData;

pub const INITIAL_CAPACITY: usize = 16;

/// A typed key-value store parameterised over key and value types.
pub struct TypedMap<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    inner: std::collections::HashMap<K, V>,
}

impl<K, V> TypedMap<K, V>
where
    K: Eq + std::hash::Hash + fmt::Debug,
    V: Clone + fmt::Debug,
{
    pub fn new() -> Self {
        TypedMap { inner: std::collections::HashMap::with_capacity(INITIAL_CAPACITY) }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }
}

impl<K, V> Default for TypedMap<K, V>
where
    K: Eq + std::hash::Hash + fmt::Debug,
    V: Clone + fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A generic result combinator.
pub trait Combinable<T, E> {
    type Output;

    fn combine<U, F>(self, other: Result<U, E>, f: F) -> Self::Output
    where
        F: FnOnce(T, U) -> T,
        E: fmt::Display;
}

/// A pipeline stage that transforms items of type `In` to type `Out`.
pub trait Stage<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    type Error: fmt::Debug;

    fn process(&mut self, input: In) -> Result<Out, Self::Error>;

    fn chain<S, Next>(self, next: S) -> Chained<Self, S>
    where
        Self: Sized,
        S: Stage<Out, Next>,
        Next: Send + 'static;
}

/// Two stages chained together.
pub struct Chained<A, B> {
    first: A,
    second: B,
}

impl<A, B, In, Mid, Out> Stage<In, Out> for Chained<A, B>
where
    A: Stage<In, Mid>,
    B: Stage<Mid, Out>,
    In: Send + 'static,
    Mid: Send + 'static,
    Out: Send + 'static,
    A::Error: Into<B::Error>,
{
    type Error = B::Error;

    fn process(&mut self, input: In) -> Result<Out, Self::Error> {
        let mid = self.first.process(input).map_err(Into::into)?;
        self.second.process(mid)
    }

    fn chain<S, Next>(self, next: S) -> Chained<Self, S>
    where
        Self: Sized,
        S: Stage<Out, Next>,
        Next: Send + 'static,
    {
        Chained { first: self, second: next }
    }
}

/// A phantom-typed handle used to associate a compile-time tag with a resource.
pub struct Tagged<T, Tag> {
    pub value: T,
    _tag: PhantomData<Tag>,
}

impl<T, Tag> Tagged<T, Tag>
where
    T: fmt::Debug,
{
    pub fn new(value: T) -> Self {
        Tagged { value, _tag: PhantomData }
    }

    pub fn map<U, F>(self, f: F) -> Tagged<U, Tag>
    where
        F: FnOnce(T) -> U,
        U: fmt::Debug,
    {
        Tagged { value: f(self.value), _tag: PhantomData }
    }
}

/// Merge two iterators of the same item type, applying a predicate.
pub fn merge_filtered<I, J, T, P>(
    left: I,
    right: J,
    predicate: P,
) -> impl Iterator<Item = T>
where
    I: Iterator<Item = T>,
    J: Iterator<Item = T>,
    T: Ord,
    P: Fn(&T) -> bool,
{
    left.chain(right).filter(move |item| predicate(item))
}

/// Higher-ranked trait bound: a callback valid for any lifetime `'a`.
pub fn apply_to_str<F>(s: &str, f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    f(s).to_owned()
}
@@CASE@@ macro_rules
macro_rules! gen {
    ($name:ident) => {
        fn $name() {}
    };
}

fn real() {}
@@CASE@@ nested
//! Demonstrates deep nesting: modules inside modules, impls inside mods,
//! functions inside impls, and constants at every level.

pub const ROOT_VERSION: &str = "2.0";

pub mod config {
    /// Maximum number of workers across all pools.
    pub const MAX_WORKERS: usize = 256;

    pub struct Settings {
        pub workers: usize,
        pub debug: bool,
    }

    impl Settings {
        pub fn default() -> Self {
            Settings { workers: 4, debug: false }
        }

        pub fn with_workers(mut self, n: usize) -> Self {
            self.workers = n;
            self
        }
    }

    pub mod logging {
        pub const DEFAULT_LEVEL: &str = "info";

        pub enum Level {
            Trace,
            Debug,
            Info,
            Warn,
            Error,
        }

        impl Level {
            pub fn from_str(s: &str) -> Option<Level> {
                match s {
                    "trace" => Some(Level::Trace),
                    "debug" => Some(Level::Debug),
                    "info" => Some(Level::Info),
                    "warn" => Some(Level::Warn),
                    "error" => Some(Level::Error),
                    _ => None,
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    Level::Trace => "trace",
                    Level::Debug => "debug",
                    Level::Info => "info",
                    Level::Warn => "warn",
                    Level::Error => "error",
                }
            }
        }

        pub trait Sink {
            fn write_record(&mut self, level: &Level, message: &str);
            fn flush(&mut self);
        }

        pub struct ConsoleSink {
            prefix: String,
        }

        impl ConsoleSink {
            pub fn new(prefix: impl Into<String>) -> Self {
                ConsoleSink { prefix: prefix.into() }
            }
        }

        impl Sink for ConsoleSink {
            fn write_record(&mut self, level: &Level, message: &str) {
                println!("[{}] {}: {}", self.prefix, level.as_str(), message);
            }

            fn flush(&mut self) {
                // nothing to flush for stdout
            }
        }
    }

    pub mod storage {
        pub const DEFAULT_PATH: &str = "/var/lib/app";
        pub const BUFFER_SIZE: usize = 8192;

        pub struct DiskStore {
            path: std::path::PathBuf,
            buf_size: usize,
        }

        impl DiskStore {
            pub fn open(path: impl Into<std::path::PathBuf>) -> std::io::Result<Self> {
                Ok(DiskStore { path: path.into(), buf_size: BUFFER_SIZE })
            }

            pub fn buf_size(&self) -> usize {
                self.buf_size
            }

            fn flush_buffers(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
    }
}

pub mod runtime {
    pub struct Handle {
        id: u64,
    }

    impl Handle {
        pub fn spawn<F>(&self, future: F)
        where
            F: std::future::Future<Output = ()> + Send + 'static,
        {
            let _ = (self.id, future);
        }

        pub fn block_on<F, T>(&self, future: F) -> T
        where
            F: std::future::Future<Output = T>,
        {
            let _ = self.id;
            let _ = future;
            unimplemented!()
        }
    }

    pub fn current() -> Handle {
        Handle { id: 0 }
    }
}
@@CASE@@ realworld
//! A realistic HTTP client module demonstrating common real-world patterns:
//! error types, builder structs, async methods, trait implementations, and
//! associated constants.

use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Maximum number of redirects the client will follow before giving up.
pub const MAX_REDIRECTS: u32 = 20;

/// Default request timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

static USER_AGENT: &str = "myapp/1.0";

/// Errors that can be returned by the HTTP client.
#[derive(Debug)]
pub enum ClientError {
    /// A network-level I/O failure.
    Io(std::io::Error),
    /// The server returned a non-2xx status code.
    Http { status: u16, body: String },
    /// The response body could not be decoded.
    Decode(String),
    /// The request timed out.
    Timeout,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Io(e) => write!(f, "io error: {e}"),
            ClientError::Http { status, body } => write!(f, "http {status}: {body}"),
            ClientError::Decode(msg) => write!(f, "decode error: {msg}"),
            ClientError::Timeout => write!(f, "request timed out"),
        }
    }
}

impl std::error::Error for ClientError {}

/// An HTTP response from the server.
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    /// Parse the body as UTF-8 text.
    pub fn text(&self) -> Result<&str, ClientError> {
        std::str::from_utf8(&self.body).map_err(|e| ClientError::Decode(e.to_string()))
    }

    /// Return true if the status code indicates success.
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
}

/// Builder for constructing an [`HttpClient`] with custom settings.
#[derive(Default)]
pub struct ClientBuilder {
    timeout: Option<Duration>,
    max_redirects: u32,
    headers: HashMap<String, String>,
}

impl ClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        ClientBuilder {
            timeout: None,
            max_redirects: MAX_REDIRECTS,
            headers: HashMap::new(),
        }
    }

    /// Override the per-request timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set a default header sent with every request.
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Consume the builder and produce an [`HttpClient`].
    pub fn build(self) -> HttpClient {
        HttpClient {
            timeout: self.timeout.unwrap_or(DEFAULT_TIMEOUT),
            max_redirects: self.max_redirects,
            default_headers: self.headers,
        }
    }
}

/// A lightweight asynchronous HTTP client.
pub struct HttpClient {
    timeout: Duration,
    max_redirects: u32,
    default_headers: HashMap<String, String>,
}

impl HttpClient {
    /// Send a GET request to `url`.
    pub async fn get(&self, url: &str) -> Result<Response, ClientError> {
        self.send("GET", url, None).await
    }

    /// Send a POST request with an optional body.
    pub async fn post(&self, url: &str, body: Option<Vec<u8>>) -> Result<Response, ClientError> {
        self.send("POST", url, body).await
    }

    /// Internal dispatch: build and execute the request.
    async fn send(
        &self,
        method: &str,
        url: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Response, ClientError> {
        let _ = (method, url, body, self.timeout, self.max_redirects);
        unimplemented!("networking backend not wired up in this fixture")
    }
}

/// A retry policy that wraps an inner client.
pub trait RetryPolicy {
    /// Maximum number of retry attempts.
    fn max_attempts(&self) -> u32;

    /// Delay before the next attempt (may be zero).
    fn delay(&self, attempt: u32) -> Duration;
}

/// Exponential back-off retry policy.
pub struct ExponentialBackoff {
    pub base_delay: Duration,
    pub max_attempts: u32,
}

impl RetryPolicy for ExponentialBackoff {
    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    fn delay(&self, attempt: u32) -> Duration {
        self.base_delay * 2u32.saturating_pow(attempt)
    }
}

/// Parse a raw `Content-Type` header value into its MIME type portion.
pub fn parse_mime_type(content_type: &str) -> &str {
    content_type.split(';').next().unwrap_or(content_type).trim()
}

/// Percent-encode a query string value.
pub fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(char::from_digit((byte >> 4) as u32, 16).unwrap_or('0'));
            out.push(char::from_digit((byte & 0xf) as u32, 16).unwrap_or('0'));
        }
    }
    out
}
@@CASE@@ sample
//! sample rust module
const MAX_RETRIES: u32 = 5;
static GREETING: &str = "hi";

pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
    async fn distance(&self, other: &Point) -> f64 {
        0.0
    }
}

pub trait Shape {
    fn area(&self) -> f64;
}

pub enum Color { Red, Green, Blue }

fn main() {
    // fn not_real() inside a comment must be ignored
    let _ = Point::new(1, 2);
}
@@CASE@@ string_literal_in_fn
fn f() {
    let s = "
fn fake() {}
";
}
@@CASE@@ string_literal_static
static S: &str = "
fn fake(x: i32) -> i32 { x }
";
fn real(y: i32) -> i32 { y }
@@CASE@@ trait_assoc_const
trait Foo {
    const BAR: usize;
}
@@CASE@@ unicode
//! Non-ASCII identifiers and string literals containing Unicode.
//! Rust allows Unicode identifiers under RFC 3101.

/// Speed of light in m/s.
pub const LICHTGESCHWINDIGKEIT: f64 = 299_792_458.0;

/// Pi to a few decimal places.
pub const KREISZAHL_PI: f64 = 3.141_592_653_589_793;

/// A point in 2-D space with coordinate names drawn from mathematics.
pub struct Koordinate {
    pub länge: f64,
    pub breite: f64,
}

impl Koordinate {
    pub fn neu(länge: f64, breite: f64) -> Self {
        Koordinate { länge, breite }
    }

    pub fn abstand(&self, andere: &Koordinate) -> f64 {
        let dx = self.länge - andere.länge;
        let dy = self.breite - andere.breite;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Trait for things that can greet in a natural language.
pub trait Grüßen {
    fn grüß_gott(&self) -> String;
    fn auf_wiedersehen(&self) -> String;
}

/// A greeter for the German locale.
pub struct DeutscherGreeter {
    pub name: String,
}

impl Grüßen for DeutscherGreeter {
    fn grüß_gott(&self) -> String {
        format!("Grüß Gott, {}!", self.name)
    }

    fn auf_wiedersehen(&self) -> String {
        format!("Auf Wiedersehen, {}!", self.name)
    }
}

/// Compute the length of a string in Unicode scalar values (not bytes).
pub fn zeichenanzahl(s: &str) -> usize {
    s.chars().count()
}

/// Reverse a string by Unicode scalar values.
pub fn umkehren(s: &str) -> String {
    s.chars().rev().collect()
}

/// A minimal Japanese-flavoured type (using romaji identifiers is also fine,
/// but this shows Unicode works end-to-end).
pub struct 辞書<K, V> {
    eintraege: Vec<(K, V)>,
}

impl<K: PartialEq, V> 辞書<K, V> {
    pub fn neu() -> Self {
        辞書 { eintraege: Vec::new() }
    }

    pub fn einfügen(&mut self, schlüssel: K, wert: V) {
        self.eintraege.push((schlüssel, wert));
    }

    pub fn suchen(&self, schlüssel: &K) -> Option<&V> {
        self.eintraege.iter().find(|(k, _)| k == schlüssel).map(|(_, v)| v)
    }
}

pub fn café_name() -> &'static str {
    // This string contains fake keywords — must be ignored.
    "fn not_real() inside a café name string; struct AlsoFake;"
}
@@CASE@@ macro_invocation_body_suppressed
my_macro! {
    fn fake() {}
    const FAKE: u32 = 42;
    static S: &str = "hi";
}
fn real() {}
@@CASE@@ local_items_in_fn_body_suppressed
pub fn outer() -> i32 {
    struct LocalStruct { x: i32 }
    fn local_fn() -> i32 { 42 }
    LocalStruct { x: local_fn() }.x
}
