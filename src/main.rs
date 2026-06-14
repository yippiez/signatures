//! `signatures` — print the signatures (functions, classes, constants) of
//! source files. Zero dependencies, standard library only.

mod cli;
mod color;
mod lang;
mod render;
mod signature;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

const USAGE: &str = "usage: signatures [--format <plain|jsonl>] [--output <truncated|full>] \
[--stream] [--no-color] [--help] [--version] <file>...";

fn main() {
    match cli::parse(std::env::args().skip(1)) {
        cli::Parsed::Help => print_help(),
        cli::Parsed::Version => println!("signatures {}", env!("CARGO_PKG_VERSION")),
        cli::Parsed::Error(msg) => {
            eprintln!("signatures: {msg}");
            eprintln!("{USAGE}");
            exit(2);
        }
        cli::Parsed::Run(cfg) => exit(run(cfg)),
    }
}

fn run(cfg: cli::Config) -> i32 {
    let colors = color::Colors { enabled: color::should_color(cfg.no_color) };
    let mut had_error = false;

    // Resolve the requested paths into a flat list of files to process.
    let mut files: Vec<PathBuf> = Vec::new();
    for p in &cfg.paths {
        if let Err(e) = collect(Path::new(p), &mut files) {
            eprintln!("signatures: {p}: {e}");
            had_error = true;
        }
    }

    let show_header = files.len() > 1;
    // In plain mode multi-file output, a blank line separates files. jsonl never
    // prints headers or blank separators.
    let plain = cfg.format == cli::Format::Plain;
    let mut out = String::new();
    let mut stdout = std::io::stdout();
    let mut first = true;

    for f in &files {
        let language = match lang::for_path(f) {
            Some(l) => l,
            None => {
                eprintln!("signatures: {}: unsupported file type", f.display());
                had_error = true;
                continue;
            }
        };
        let source = match std::fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("signatures: {}: {e}", f.display());
                had_error = true;
                continue;
            }
        };
        // Strip a leading UTF-8 BOM so the first declaration is not hidden
        // behind the byte-order mark.
        let source = source.strip_prefix('\u{feff}').unwrap_or(&source);

        let sigs = language.extract(source);
        let path = f.display().to_string();
        let full = cfg.output == cli::OutputMode::Full;

        if cfg.stream {
            // Incremental path: write+flush after each emitted unit. Must be
            // byte-identical to the buffered path below.
            if plain && !first {
                let _ = stdout.write_all(b"\n");
                let _ = stdout.flush();
            }
            if plain && show_header {
                let mut hdr = colors.header(&path);
                hdr.push('\n');
                let _ = stdout.write_all(hdr.as_bytes());
                let _ = stdout.flush();
            }
            if full {
                // Full mode: emit the outermost decls (coverage rule), flushing
                // per block, with one blank line between consecutive plain blocks.
                let lines: Vec<&str> = source.lines().collect();
                let emit = render::full_mode_emit(&sigs);
                let lk = render::full_line_kinds(&sigs);
                let mut first_block = true;
                for (s, &keep) in sigs.iter().zip(emit.iter()) {
                    if !keep {
                        continue;
                    }
                    if let Some(block) =
                        render::render_one_full(&path, s, &lines, cfg.format, &colors, &lk)
                    {
                        let mut chunk = String::new();
                        if plain && !first_block {
                            chunk.push('\n');
                        }
                        chunk.push_str(&block);
                        chunk.push('\n');
                        let _ = stdout.write_all(chunk.as_bytes());
                        let _ = stdout.flush();
                        first_block = false;
                    }
                }
            } else {
                for s in &sigs {
                    let mut line = render::render_one(&path, s, &colors, cfg.format, cfg.output);
                    line.push('\n');
                    let _ = stdout.write_all(line.as_bytes());
                    let _ = stdout.flush();
                }
            }
        } else {
            if plain && !first {
                out.push('\n');
            }
            if full {
                let lines: Vec<&str> = source.lines().collect();
                render::render_full(&path, &sigs, &lines, &colors, show_header, cfg.format, &mut out);
            } else {
                render::render(&path, &sigs, &colors, show_header, cfg.format, cfg.output, &mut out);
            }
        }
        first = false;
    }

    if !cfg.stream {
        let _ = stdout.write_all(out.as_bytes());
        let _ = stdout.flush();
    }

    if had_error {
        1
    } else {
        0
    }
}

/// Resolve a path into files. A file is added as-is (so unsupported extensions
/// can be reported); a directory is walked recursively, keeping only files of a
/// supported language.
fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let meta = std::fs::metadata(path)?;
    if meta.is_dir() {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        entries.sort();
        for e in entries {
            if e.is_dir() {
                // Ignore errors on individual subdirectories.
                let _ = collect(&e, files);
            } else if lang::for_path(&e).is_some() {
                files.push(e);
            }
        }
    } else {
        files.push(path.to_path_buf());
    }
    Ok(())
}

fn print_help() {
    println!("{USAGE}");
    println!();
    println!("Print the signatures (functions, classes, constants) of source files.");
    println!();
    println!("Options:");
    println!("  --format <plain|jsonl>      output format (default: plain). jsonl emits one");
    println!("                              JSON object per signature per line (no color).");
    println!("  --output <truncated|full>   detail level (default: truncated). full prints the");
    println!("                              complete verbatim source of each top-level decl");
    println!("                              (bodies included; nested members shown inside their");
    println!("                              parent; not colorized).");
    println!("  --stream                    stream each finding as produced (flush per line)");
    println!("  --no-color    disable ANSI colors (colors are on by default)");
    println!("  -h, --help    show this help");
    println!("      --version show version");
    println!();
    println!(
        "Supported languages: Python (.py, .pyi), Rust (.rs), Go (.go), \
         JavaScript (.js, .mjs, .cjs, .jsx), TypeScript (.ts, .tsx, .mts, .cts), \
         Java (.java), C (.c, .h), C++ (.cpp, .cc, .cxx, .hpp, .hh, .hxx), \
         C# (.cs, .csx), Kotlin (.kt, .kts), Swift (.swift), \
         PHP (.php, .phtml), Scala (.scala, .sc), Dart (.dart), \
         Ruby (.rb, .rake, .gemspec), Lua (.lua), Bash (.sh, .bash)"
    );
}
