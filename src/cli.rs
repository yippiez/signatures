//! Hand-rolled argument parsing (no external crates).

pub struct Config {
    pub paths: Vec<String>,
    pub no_color: bool,
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
    let mut positional_only = false;

    for a in args {
        if positional_only {
            paths.push(a);
            continue;
        }
        match a.as_str() {
            "--no-color" => no_color = true,
            "-h" | "--help" => return Parsed::Help,
            "--version" => return Parsed::Version,
            "--" => positional_only = true,
            s if s.starts_with('-') && s != "-" => {
                return Parsed::Error(format!("unknown option: {s}"));
            }
            _ => paths.push(a),
        }
    }

    if paths.is_empty() {
        return Parsed::Error("no input files".to_string());
    }
    Parsed::Run(Config { paths, no_color })
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
}
