use super::types_tools::ToolResult;
use serde_json::Value;

pub(super) fn from_args(args: &Value) -> Result<String, ToolResult> {
    let prompt = args["prompt"]
        .as_str()
        .ok_or_else(|| ToolResult::err("Paramètre 'prompt' manquant ou vide"))?;
    if prompt.trim().is_empty() {
        return Err(ToolResult::err("Paramètre 'prompt' manquant ou vide"));
    }
    if prompt.chars().count() > super::subagent_instruction_delivery::MAX_PROMPT_SIZE {
        return Err(ToolResult::err("Prompt sous-agent trop long."));
    }
    Ok(prompt.to_string())
}
