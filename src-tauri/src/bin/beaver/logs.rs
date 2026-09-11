use crate::output::Out;
use std::path::{Path, PathBuf};

const DEFAULT_LINES: usize = 50;
const MAX_OUTPUT_LINE_CHARS: usize = 16 * 1024;

#[derive(Debug, Eq, PartialEq)]
struct Options {
    source: &'static str,
    lines: usize,
}

#[derive(Debug, Eq, PartialEq)]
enum ParseError {
    Source,
    Lines,
    Usage,
}

fn sources(root: &Path) -> Vec<(&'static str, PathBuf)> {
    vec![
        ("app", root.join("logs/beaver.log")),
        ("gateway", root.join("logs/gateway-audit.jsonl")),
        ("reveils", root.join("logs/wakeups.jsonl")),
    ]
}

fn known_source(value: &str) -> Option<&'static str> {
    ["app", "gateway", "reveils"]
        .into_iter()
        .find(|source| *source == value)
}

fn parse_lines(value: &str) -> Result<usize, ParseError> {
    value
        .parse::<usize>()
        .ok()
        .filter(|lines| (1..=crate::tail::MAX_LINES).contains(lines))
        .ok_or(ParseError::Lines)
}

fn parse_args(args: &[String]) -> Result<Options, ParseError> {
    match args {
        [] => Ok(Options {
            source: "app",
            lines: DEFAULT_LINES,
        }),
        [source] => Ok(Options {
            source: known_source(source).ok_or(ParseError::Source)?,
            lines: DEFAULT_LINES,
        }),
        [flag, count] if flag == "-n" => Ok(Options {
            source: "app",
            lines: parse_lines(count)?,
        }),
        [source, flag, count] if flag == "-n" => Ok(Options {
            source: known_source(source).ok_or(ParseError::Source)?,
            lines: parse_lines(count)?,
        }),
        [source, ..] if !source.starts_with('-') && known_source(source).is_none() => {
            Err(ParseError::Source)
        }
        _ => Err(ParseError::Usage),
    }
}

fn safe_log_line(line: &str) -> String {
    line.chars()
        .take(MAX_OUTPUT_LINE_CHARS)
        .map(|character| {
            if character.is_control() {
                '?'
            } else {
                character
            }
        })
        .collect()
}

pub fn run(out: &Out, args: &[String]) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(ParseError::Source) => {
            out.line(out.t(
                "Source inconnue. Sources : app, gateway, reveils.",
                "Unknown source. Sources: app, gateway, reveils.",
            ));
            return 2;
        }
        Err(ParseError::Lines) => {
            out.line(out.t(
                "Le nombre de lignes doit être compris entre 1 et 5000.",
                "Line count must be between 1 and 5000.",
            ));
            return 2;
        }
        Err(ParseError::Usage) => {
            out.line(out.t(
                "Usage : beaver logs [app|gateway|reveils] [-n N]",
                "Usage: beaver logs [app|gateway|reveils] [-n N]",
            ));
            return 2;
        }
    };
    let root = cl_go_dash_lib::cli_support::data_dir();
    let path = sources(&root)
        .into_iter()
        .find_map(|(source, path)| (source == options.source).then_some(path))
        .expect("validated source");
    match crate::tail::last_lines(&path, options.lines) {
        Ok(lines) => {
            for line in lines {
                out.line(&safe_log_line(&line));
            }
            0
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            out.line(out.t("Journal pas encore créé.", "Log not created yet."));
            0
        }
        Err(_) => {
            out.line(out.t("Journal illisible.", "Log cannot be read."));
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn options_par_defaut() {
        let options = parse_args(&[]).expect("defaults");
        assert_eq!(options.source, "app");
        assert_eq!(options.lines, 50);
    }

    #[test]
    fn source_et_nombre_valides() {
        let options = parse_args(&args(&["reveils", "-n", "10"])).expect("options");
        assert_eq!(options.source, "reveils");
        assert_eq!(options.lines, 10);
    }

    #[test]
    fn nombre_est_strict_et_borne() {
        assert!(matches!(
            parse_args(&args(&["-n", "0"])),
            Err(ParseError::Lines)
        ));
        assert!(matches!(
            parse_args(&args(&["-n", "5001"])),
            Err(ParseError::Lines)
        ));
        assert!(matches!(
            parse_args(&args(&["-n", "10x"])),
            Err(ParseError::Lines)
        ));
    }

    #[test]
    fn source_inconnue_est_distinguee() {
        assert!(matches!(
            parse_args(&args(&["inconnu"])),
            Err(ParseError::Source)
        ));
    }

    #[test]
    fn syntaxe_invalide_est_distinguee() {
        assert!(matches!(
            parse_args(&args(&["app", "--bad", "10"])),
            Err(ParseError::Usage)
        ));
    }

    #[test]
    fn sortie_ne_peut_pas_injecter_le_terminal() {
        assert_eq!(safe_log_line("ok\u{1b}[31m"), "ok?[31m");
        assert_eq!(
            safe_log_line(&"é".repeat(MAX_OUTPUT_LINE_CHARS + 1))
                .chars()
                .count(),
            MAX_OUTPUT_LINE_CHARS
        );
    }
}
