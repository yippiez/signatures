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
