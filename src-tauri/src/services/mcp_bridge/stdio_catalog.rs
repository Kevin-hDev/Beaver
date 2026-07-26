pub const IMESSAGE_CONNECTOR_ID: &str = "imessage";
pub const IMESSAGE_INSTALL_COMMAND: &str =
    "deno run --allow-read --allow-env --allow-sys --allow-ffi jsr:@wyattjoh/imessage-mcp@0.4.2";
pub const IMESSAGE_ARGS: &[&str] = &[
    "run",
    "--allow-read",
    "--allow-env",
    "--allow-sys",
    "--allow-ffi",
    "jsr:@wyattjoh/imessage-mcp@0.4.2",
];

pub fn command_parts(connector_id: &str) -> Option<(&'static str, &'static [&'static str])> {
    match connector_id {
        "context7" => Some(("npx", &["@upstash/context7-mcp@2.2.5"])),
        "huggingface" => Some(("npx", &["@llmindset/hf-mcp-server@0.3.13"])),
        "producthunt" => Some(("uvx", &["product-hunt-mcp==0.1.0"])),
        "reddit" => Some(("npx", &["reddit-mcp-server@1.4.5"])),
        _ => None,
    }
}

pub fn install_command(connector_id: &str) -> Option<String> {
    if connector_id == IMESSAGE_CONNECTOR_ID {
        return Some(IMESSAGE_INSTALL_COMMAND.to_string());
    }
    let (program, args) = command_parts(connector_id)?;
    Some(format!("{program} {}", args.join(" ")))
}
