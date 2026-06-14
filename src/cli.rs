//! Hand-rolled argument parsing (no external crates).

/// Output serialization format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Human-readable, colorized, indented text (default; current behavior).
    Plain,
    /// One compact JSON object per signature per line (JSON Lines).
    Jsonl,
}

/// How much of an eliding declaration's value to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Keep the elided `…` value (default; current behavior).
    Truncated,
    /// Expand to the full code part instead of the `…` elision.
    Full,
}

pub struct Config {
    pub paths: Vec<String>,
    pub no_color: bool,
    pub format: Format,
    pub output: OutputMode,
    pub stream: bool,
}

pub enum Parsed {
    Run(Config),
    Help,
    Version,
    Error(String),
}

/// Parse the program arguments (excluding argv[0]).
pub fn parse<I: Iterator<Item = String>>(args: I) -> Parsed {
    let mut paths = Vec::new();
    let mut no_color = false;
    let mut format = Format::Plain;
    let mut output = OutputMode::Truncated;
    let mut stream = false;
    let mut positional_only = false;

    // Iterate manually so flags that take a separate value (e.g. `--format jsonl`)
    // can consume the following argument.
    let mut it = args;
    while let Some(a) = it.next() {
        if positional_only {
            paths.push(a);
            continue;
        }
        match a.as_str() {
            "--no-color" => no_color = true,
            "--stream" => stream = true,
            "-h" | "--help" => return Parsed::Help,
            "--version" => return Parsed::Version,
            "--" => positional_only = true,
            "--format" => match it.next() {
                Some(v) => match parse_format(&v) {
                    Some(f) => format = f,
                    None => return Parsed::Error(format!("invalid value for --format: {v}")),
                },
                None => return Parsed::Error("--format requires a value".to_string()),
            },
            "--output" => match it.next() {
                Some(v) => match parse_output(&v) {
                    Some(o) => output = o,
                    None => return Parsed::Error(format!("invalid value for --output: {v}")),
                },
                None => return Parsed::Error("--output requires a value".to_string()),
            },
            s if s.starts_with("--format=") => {
                let v = &s["--format=".len()..];
                match parse_format(v) {
                    Some(f) => format = f,
                    None => return Parsed::Error(format!("invalid value for --format: {v}")),
                }
            }
            s if s.starts_with("--output=") => {
                let v = &s["--output=".len()..];
                match parse_output(v) {
                    Some(o) => output = o,
                    None => return Parsed::Error(format!("invalid value for --output: {v}")),
                }
            }
            s if s.starts_with('-') && s != "-" => {
                return Parsed::Error(format!("unknown option: {s}"));
            }
            _ => paths.push(a),
        }
    }

    if paths.is_empty() {
        return Parsed::Error("no input files".to_string());
    }
    Parsed::Run(Config { paths, no_color, format, output, stream })
}

fn parse_format(v: &str) -> Option<Format> {
    match v {
        "plain" => Some(Format::Plain),
        "jsonl" => Some(Format::Jsonl),
        _ => None,
    }
}

fn parse_output(v: &str) -> Option<OutputMode> {
    match v {
        "truncated" => Some(OutputMode::Truncated),
        "full" => Some(OutputMode::Full),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Parsed {
        parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn collects_paths_and_flag() {
        match run(&["--no-color", "a.py", "b.py"]) {
            Parsed::Run(c) => {
                assert!(c.no_color);
                assert_eq!(c.paths, vec!["a.py", "b.py"]);
                assert_eq!(c.format, Format::Plain);
                assert_eq!(c.output, OutputMode::Truncated);
                assert!(!c.stream);
            }
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn unknown_flag_errors() {
        assert!(matches!(run(&["--bogus", "a.py"]), Parsed::Error(_)));
    }

    #[test]
    fn no_files_errors() {
        assert!(matches!(run(&["--no-color"]), Parsed::Error(_)));
    }

    #[test]
    fn double_dash_treats_rest_as_paths() {
        match run(&["--", "--weird-name.py"]) {
            Parsed::Run(c) => assert_eq!(c.paths, vec!["--weird-name.py"]),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn format_space_value() {
        match run(&["--format", "jsonl", "a.py"]) {
            Parsed::Run(c) => assert_eq!(c.format, Format::Jsonl),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn format_eq_value() {
        match run(&["--format=plain", "a.py"]) {
            Parsed::Run(c) => assert_eq!(c.format, Format::Plain),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn output_space_and_eq() {
        match run(&["--output", "full", "a.py"]) {
            Parsed::Run(c) => assert_eq!(c.output, OutputMode::Full),
            _ => panic!("expected Run"),
        }
        match run(&["--output=truncated", "a.py"]) {
            Parsed::Run(c) => assert_eq!(c.output, OutputMode::Truncated),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn stream_flag() {
        match run(&["--stream", "a.py"]) {
            Parsed::Run(c) => assert!(c.stream),
            _ => panic!("expected Run"),
        }
    }

    #[test]
    fn invalid_format_errors() {
        assert!(matches!(run(&["--format", "xml", "a.py"]), Parsed::Error(_)));
        assert!(matches!(run(&["--format=xml", "a.py"]), Parsed::Error(_)));
    }

    #[test]
    fn invalid_output_errors() {
        assert!(matches!(run(&["--output", "partial", "a.py"]), Parsed::Error(_)));
        assert!(matches!(run(&["--output=partial", "a.py"]), Parsed::Error(_)));
    }

    #[test]
    fn missing_value_errors() {
        assert!(matches!(run(&["--format"]), Parsed::Error(_)));
        assert!(matches!(run(&["--output"]), Parsed::Error(_)));
    }

    #[test]
    fn value_after_double_dash_is_path() {
        // `--` makes everything positional, so `--format` here is a path.
        match run(&["--", "--format", "jsonl"]) {
            Parsed::Run(c) => {
                assert_eq!(c.paths, vec!["--format", "jsonl"]);
                assert_eq!(c.format, Format::Plain);
            }
            _ => panic!("expected Run"),
        }
    }
}
