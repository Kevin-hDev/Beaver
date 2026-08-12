use super::types_tools::ShellOutput;
use std::path::Path;

pub fn validate(command: &str, working_dir: &Path) -> Result<(), String> {
    let tokens = parse(command)?;
    validate_tokens(&tokens, working_dir)
}

pub async fn execute(
    command: &str,
    working_dir: &Path,
    timeout_secs: Option<u64>,
    cancel: tokio_util::sync::CancellationToken,
    work: super::agent_work_supervision::ShellWork,
) -> Result<ShellOutput, String> {
    let tokens = parse(command)?;
    validate_tokens(&tokens, working_dir)?;
    let admission = work
        .try_admit()
        .map_err(|error| error.public_code().to_string())?;
    let working_dir = working_dir.to_path_buf();
    let request_cancel = cancel.clone();
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();
    admission
        .spawn(move |shutdown| async move {
            let task =
                super::subagent_explorer_process::run(&tokens, &working_dir, timeout_secs, cancel);
            tokio::pin!(task);
            let result = tokio::select! {
                _ = shutdown.cancelled() => {
                    request_cancel.cancel();
                    task.await
                }
                result = &mut task => result,
            };
            let _ = result_tx.send(result);
        })
        .map_err(|error| error.public_code().to_string())?;
    result_rx
        .await
        .map_err(|_| "Commande d'exploration indisponible.".to_string())?
}

fn parse(command: &str) -> Result<Vec<String>, String> {
    let command = command.trim();
    if command.is_empty()
        || command.contains([';', '|', '>', '<', '`', '\n', '\r', '\'', '"', '\\'])
        || command.contains("$(")
        || command.contains("&&")
        || command.contains("||")
    {
        return Err("Commande d'exploration refusée.".to_string());
    }
    let tokens = command
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if tokens.len() > 32 {
        return Err("Commande d'exploration trop longue.".to_string());
    }
    Ok(tokens)
}

fn validate_tokens(tokens: &[String], working_dir: &Path) -> Result<(), String> {
    let Some(program) = tokens.first().map(String::as_str) else {
        return Err("Commande d'exploration refusée.".to_string());
    };
    match program {
        "pwd" if tokens.len() == 1 => Ok(()),
        "ls" => validate_command_paths("ls", &tokens[1..], working_dir),
        "tree" => validate_tree(&tokens[1..], working_dir),
        "file" | "stat" | "wc" | "du" | "df" => {
            validate_command_paths(program, &tokens[1..], working_dir)
        }
        "git" => validate_git(&tokens[1..], working_dir),
        _ => Err("Commande d'exploration refusée.".to_string()),
    }
}

fn validate_tree(args: &[String], working_dir: &Path) -> Result<(), String> {
    if args.len() < 2 || args[0] != "-L" {
        return Err("tree exige -L avec une profondeur de 1 à 8.".to_string());
    }
    let depth = args[1]
        .parse::<u8>()
        .map_err(|_| "Profondeur invalide.".to_string())?;
    if !(1..=8).contains(&depth) {
        return Err("Profondeur invalide.".to_string());
    }
    if args[2..].iter().any(|arg| arg.starts_with('-')) {
        return Err("Option tree refusée.".to_string());
    }
    validate_paths(&args[2..], working_dir)
}

fn validate_command_paths(
    program: &str,
    args: &[String],
    working_dir: &Path,
) -> Result<(), String> {
    super::subagent_explorer_bash_options::validate(program, args)?;
    validate_paths(args, working_dir)
}

fn validate_paths(args: &[String], working_dir: &Path) -> Result<(), String> {
    for arg in args.iter().filter(|arg| !arg.starts_with('-')) {
        super::subagent_tool_guard::validate_path_argument(
            Some(&serde_json::Value::String(arg.clone())),
            working_dir,
        )?;
    }
    Ok(())
}

fn validate_git(args: &[String], working_dir: &Path) -> Result<(), String> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err("Sous-commande Git requise.".to_string());
    };
    match subcommand {
        "status" | "diff" | "log" | "show" | "rev-parse" | "ls-files" => {
            super::subagent_explorer_bash_options::validate_git(subcommand, &args[1..])?;
            validate_paths(&args[1..], working_dir)
        }
        "remote" if args.get(1).map(String::as_str) == Some("-v") && args.len() == 2 => Ok(()),
        "tag" if args.get(1).map(String::as_str) == Some("--list") && args.len() == 2 => Ok(()),
        "branch"
            if args[1..].iter().all(|arg| {
                matches!(
                    arg.as_str(),
                    "-a" | "--all"
                        | "-r"
                        | "--remotes"
                        | "-v"
                        | "-vv"
                        | "--list"
                        | "--show-current"
                )
            }) =>
        {
            Ok(())
        }
        _ => Err("Commande Git non informative refusée.".to_string()),
    }
}
