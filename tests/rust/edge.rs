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
