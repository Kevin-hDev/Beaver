use super::stdio_catalog;

const ALLOWED_PROGRAMS: &[&str] = &["npx", "uvx", "deno"];

static ARG_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9@/_.:=\-]+$").unwrap());

pub struct ParsedCommand {
    pub program: String,
    pub args: Vec<String>,
}

pub fn parse_install_command(connector_id: &str, raw: &str) -> Result<ParsedCommand, String> {
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.is_empty() {
        return Err("commande vide".to_string());
    }

    let program = parts[0];
    if !ALLOWED_PROGRAMS.contains(&program) {
        return Err("programme non autorisé".to_string());
    }

    let regex = &*ARG_REGEX;
    let mut args = Vec::new();

    for part in &parts[1..] {
        if !regex.is_match(part) {
            return Err("argument invalide".to_string());
        }
        args.push(part.to_string());
    }

    if connector_id == stdio_catalog::IMESSAGE_CONNECTOR_ID
        && (program != "deno" || args != stdio_catalog::IMESSAGE_ARGS)
    {
        return Err("commande iMessage non autorisée".to_string());
    }

    if args.is_empty() {
        return Err("aucun package spécifié".to_string());
    }

    validate_catalog_command(connector_id, program, &args)?;

    Ok(ParsedCommand {
        program: program.to_string(),
        args,
    })
}

fn validate_catalog_command(
    connector_id: &str,
    program: &str,
    args: &[String],
) -> Result<(), String> {
    if connector_id == stdio_catalog::IMESSAGE_CONNECTOR_ID {
        return Ok(());
    }
    let Some((expected_program, expected_args)) = stdio_catalog::command_parts(connector_id) else {
        return Err("connecteur stdio non autorisé".to_string());
    };
    if program != expected_program || args.len() != expected_args.len() {
        return Err("commande MCP non autorisée".to_string());
    }
    if args
        .iter()
        .zip(expected_args)
        .any(|(actual, expected)| actual != expected)
    {
        return Err("commande MCP non autorisée".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npx_valid() {
        let cmd = parse_install_command("context7", "npx @upstash/context7-mcp@2.2.5").unwrap();
        assert_eq!(cmd.program, "npx");
        assert_eq!(cmd.args, vec!["@upstash/context7-mcp@2.2.5"]);
    }

    #[test]
    fn test_parse_uvx_valid() {
        let cmd = parse_install_command("producthunt", "uvx product-hunt-mcp==0.1.0").unwrap();
        assert_eq!(cmd.program, "uvx");
        assert_eq!(cmd.args, vec!["product-hunt-mcp==0.1.0"]);
    }

    #[test]
    fn test_parse_imessage_deno_valid() {
        let cmd = parse_install_command(
            "imessage",
            "deno run --allow-read --allow-env --allow-sys --allow-ffi jsr:@wyattjoh/imessage-mcp@0.4.2",
        )
        .unwrap();
        assert_eq!(cmd.program, "deno");
        assert_eq!(
            cmd.args,
            vec![
                "run",
                "--allow-read",
                "--allow-env",
                "--allow-sys",
                "--allow-ffi",
                "jsr:@wyattjoh/imessage-mcp@0.4.2"
            ]
        );
    }

    #[test]
    fn test_reject_deno_ffi() {
        assert!(parse_install_command("other", "deno run --allow-ffi jsr:@pkg/mcp").is_err());
    }

    #[test]
    fn test_reject_imessage_extra_write() {
        assert!(parse_install_command(
            "imessage",
            "deno run --allow-read --allow-write --allow-env --allow-sys --allow-ffi jsr:@wyattjoh/imessage-mcp@0.4.2"
        )
        .is_err());
    }

    #[test]
    fn test_reject_deno_allow_all() {
        assert!(parse_install_command("other", "deno run -A jsr:@pkg/mcp").is_err());
    }

    #[test]
    fn test_reject_shell() {
        assert!(parse_install_command("other", "bash -c 'rm -rf /'").is_err());
    }

    #[test]
    fn test_reject_pipe() {
        assert!(parse_install_command("other", "npx foo | cat /etc/passwd").is_err());
    }

    #[test]
    fn test_reject_semicolon() {
        assert!(parse_install_command("other", "npx foo; rm -rf /").is_err());
    }

    #[test]
    fn test_reject_empty() {
        assert!(parse_install_command("other", "").is_err());
    }

    #[test]
    fn test_reject_no_args() {
        assert!(parse_install_command("other", "npx").is_err());
    }

    #[test]
    fn test_reject_backtick() {
        assert!(parse_install_command("other", "npx `whoami`").is_err());
    }

    #[test]
    fn test_reject_dollar() {
        assert!(parse_install_command("other", "npx $(cat /etc/passwd)").is_err());
    }

    #[test]
    fn test_reject_unknown_npx_package() {
        assert!(parse_install_command("other", "npx safe-looking-package").is_err());
    }

    #[test]
    fn test_reject_catalog_command_with_extra_flag() {
        assert!(
            parse_install_command("context7", "npx --yes @upstash/context7-mcp@2.2.5").is_err()
        );
    }
}
