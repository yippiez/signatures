//! Minimal ANSI coloring, no dependencies.

use std::io::IsTerminal;

const RESET: &str = "\x1b[0m";

/// Holds whether color is active and wraps text in SGR codes accordingly.
pub struct Colors {
    pub enabled: bool,
}

impl Colors {
    fn wrap(&self, code: &str, s: &str) -> String {
        if self.enabled && !s.is_empty() {
            format!("\x1b[{code}m{s}{RESET}")
        } else {
            s.to_string()
        }
    }

    /// `def` / `class` / `async` keywords — bold magenta.
    pub fn kw(&self, s: &str) -> String {
        self.wrap("1;35", s)
    }

    /// Function / class names — bold cyan.
    pub fn name(&self, s: &str) -> String {
        self.wrap("1;36", s)
    }

    /// Constant names — green.
    pub fn constant(&self, s: &str) -> String {
        self.wrap("32", s)
    }

    /// Parameters, punctuation, elided values — dim.
    pub fn dim(&self, s: &str) -> String {
        self.wrap("2", s)
    }

    /// File headers — bold underline.
    pub fn header(&self, s: &str) -> String {
        self.wrap("1;4", s)
    }
}

/// Decide whether to emit color. Off when `--no-color` is given, when the
/// `NO_COLOR` env var is set (https://no-color.org), or when stdout is not a TTY.
pub fn should_color(no_color_flag: bool) -> bool {
    if no_color_flag || std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    std::io::stdout().is_terminal()
}
