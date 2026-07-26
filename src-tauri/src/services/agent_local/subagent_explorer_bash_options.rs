const ERROR: &str = "Option d'exploration refusée.";

pub fn validate(program: &str, args: &[String]) -> Result<(), String> {
    match program {
        "ls" => validate_options(args, "aAlh1dF", &[
            "--all",
            "--almost-all",
            "--human-readable",
            "--directory",
            "--classify",
        ], false),
        "file" => validate_options(args, "b", &["--brief"], false),
        "stat" => validate_options(args, "", &[], false),
        "wc" => validate_options(args, "lwcm", &["--lines", "--words", "--bytes", "--chars"], false),
        "du" => validate_options(args, "hsa", &["--human-readable", "--summarize", "--all"], false),
        "df" => validate_options(args, "h", &["--human-readable"], false),
        _ => Err(ERROR.into()),
    }
}

pub fn validate_git(subcommand: &str, args: &[String]) -> Result<(), String> {
    match subcommand {
        "status" => validate_options(
            args,
            "sb",
            &["--short", "--branch", "--porcelain", "--untracked-files=no", "--untracked-files=normal", "--untracked-files=all"],
            false,
        ),
        "diff" => validate_options(
            args,
            "",
            &["--stat", "--shortstat", "--numstat", "--name-only", "--name-status", "--summary", "--check", "--cached", "--staged"],
            false,
        ),
        "log" => validate_options(
            args,
            "",
            &["--oneline", "--stat", "--shortstat", "--name-only", "--name-status", "--decorate", "--no-decorate", "--all", "--graph"],
            true,
        ),
        "show" => validate_options(
            args,
            "",
            &["--oneline", "--stat", "--shortstat", "--name-only", "--name-status", "--decorate", "--no-decorate"],
            false,
        ),
        "rev-parse" => validate_options(
            args,
            "",
            &["--show-toplevel", "--show-prefix", "--show-cdup", "--git-dir", "--is-inside-work-tree"],
            false,
        ),
        "ls-files" => validate_options(
            args,
            "cmdos",
            &["--cached", "--modified", "--deleted", "--others", "--stage", "--exclude-standard"],
            false,
        ),
        _ => Err(ERROR.into()),
    }
}

fn validate_options(
    args: &[String],
    short: &str,
    long: &[&str],
    numeric_short: bool,
) -> Result<(), String> {
    for arg in args {
        if arg == "--" || !arg.starts_with('-') || arg == "-" {
            continue;
        }
        let flags = arg.strip_prefix('-').unwrap_or_default();
        if numeric_short && !flags.is_empty() && flags.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        if arg.starts_with("--") {
            if long.contains(&arg.as_str()) {
                continue;
            }
            return Err(ERROR.into());
        }
        if flags.chars().all(|flag| short.contains(flag)) {
            continue;
        }
        return Err(ERROR.into());
    }
    Ok(())
}
