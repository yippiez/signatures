//! Language registry. Adding a language means writing one module that
//! implements [`Language`] and wiring it into [`for_path`] — nothing else.

use std::path::Path;

use crate::signature::Signature;

mod braces;
mod python;
mod ruby;
mod lua;
mod bash;

/// A source language able to extract signatures from a file's contents.
pub trait Language {
    fn extract(&self, source: &str) -> Vec<Signature>;
}

/// Pick a language for a path based on its extension, or `None` if unsupported.
pub fn for_path(path: &Path) -> Option<Box<dyn Language>> {
    let ext = path.extension().and_then(|e| e.to_str())?;
    use braces::{BraceLang, Lang};
    match ext {
        "py" | "pyi" => Some(Box::new(python::PythonLang)),
        "rs" => Some(Box::new(BraceLang { lang: Lang::Rust })),
        "go" => Some(Box::new(BraceLang { lang: Lang::Go })),
        "js" | "mjs" | "cjs" | "jsx" => Some(Box::new(BraceLang { lang: Lang::Js })),
        "ts" | "tsx" | "mts" | "cts" => Some(Box::new(BraceLang { lang: Lang::Ts })),
        "java" => Some(Box::new(BraceLang { lang: Lang::Java })),
        "c" | "h" => Some(Box::new(BraceLang { lang: Lang::C })),
        "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" | "ipp" | "tpp" => {
            Some(Box::new(BraceLang { lang: Lang::Cpp }))
        }
        "cs" | "csx" => Some(Box::new(BraceLang { lang: Lang::CSharp })),
        "kt" | "kts" => Some(Box::new(BraceLang { lang: Lang::Kotlin })),
        "swift" => Some(Box::new(BraceLang { lang: Lang::Swift })),
        "php" | "phtml" | "php3" | "php4" | "php5" | "phps" => {
            Some(Box::new(BraceLang { lang: Lang::Php }))
        }
        "scala" | "sc" => Some(Box::new(BraceLang { lang: Lang::Scala })),
        "dart" => Some(Box::new(BraceLang { lang: Lang::Dart })),
        "rb" | "rake" | "gemspec" => Some(Box::new(ruby::RubyLang)),
        "lua" => Some(Box::new(lua::LuaLang)),
        "sh" | "bash" => Some(Box::new(bash::BashLang)),
        _ => None,
    }
}
