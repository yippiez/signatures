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
