use super::protocol::HostToolResult;
use crate::services::agent_local::tool_artifact::PendingArtifact;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;

pub(super) fn to_tool_result(result: Result<HostToolResult, String>) -> ToolResult {
    match result {
        Ok(result) => {
            let (content, pending_artifacts) = match result.content {
                super::tool_result_contract::ToolResultContent::Text(content) => (content, Vec::new()),
                super::tool_result_contract::ToolResultContent::Blocks(blocks) => {
                    let content = super::tool_result_contract::ToolResultContent::Blocks(blocks);
                    if super::tool_result_contract::validate(&content).is_err() {
                        return ToolResult::unavailable(
                            super::error_codes::RESULT_INVALID,
                            "Résultat d'extension indisponible.",
                            false,
                        );
                    }
                    rich_content(content, result.is_error)
                }
            };
            let mut tool_result = if result.is_error {
                ToolResult::error(content, "extension_tool_error", ToolErrorCategory::External, false)
            } else { ToolResult::ok(content) };
            if tool_result.set_pending_artifacts(pending_artifacts).is_err() {
                return ToolResult::unavailable(
                    super::error_codes::RESULT_INVALID,
                    "Résultat d'extension indisponible.",
                    false,
                );
            }
            tool_result.mark_truncated(result.truncated);
            if let Some(summary) = result.display_summary { tool_result = tool_result.with_display_summary(summary); }
            tool_result
        }
        Err(_) => ToolResult::error("L'extension n'a pas pu confirmer le résultat de cet outil.", super::error_codes::HOST_UNAVAILABLE, ToolErrorCategory::External, false)
            .with_error_hint("Vérifier l'état du projet ou du service avant de relancer : l'action a pu être exécutée."),
    }
}

fn rich_content(
    content: super::tool_result_contract::ToolResultContent,
    is_error: bool,
) -> (String, Vec<PendingArtifact>) {
    let super::tool_result_contract::ToolResultContent::Blocks(blocks) = content else {
        unreachable!("rich content is validated as blocks before conversion");
    };
    let mut text = String::new();
    let mut pending_artifacts = Vec::new();
    for block in blocks {
        match block {
            super::tool_result_contract::ToolResultBlock::Text { text: block_text } => {
                text.push_str(&block_text);
            }
            super::tool_result_contract::ToolResultBlock::File {
                path,
                purpose,
                display_name,
            } if !is_error => {
                let visible_name = display_name.as_deref().unwrap_or(&path);
                text.push_str(&format!("[{}: {visible_name}]", purpose.as_str()));
                pending_artifacts.push(PendingArtifact::from_validated(
                    path,
                    display_name,
                    purpose,
                ));
            }
            // A declared file cannot make an error result look successful or
            // authorize a read after the extension reported failure.
            super::tool_result_contract::ToolResultBlock::File { .. } => {}
        }
    }
    (text, pending_artifacts)
}

pub(super) fn extension_context_unavailable() -> ToolResult {
    ToolResult::error(
        "Contexte d'extension indisponible.",
        "extension_context_unavailable",
        ToolErrorCategory::Unavailable,
        false,
    )
}

#[cfg(test)]
#[path = "runtime_dispatch_result_tests.rs"]
mod tests;
