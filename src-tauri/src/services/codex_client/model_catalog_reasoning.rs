use super::CatalogModel;
use crate::services::codex_client::types::CodexRequest;

/// Ultra is a local Codex mode, not a literal Responses API effort.
/// Match Codex rust-v0.153.4/client.rs::reasoning_effort_for_request (2026-09-07).
/// Keep the session/continuation identity unchanged; only translate the wire request.
pub(crate) async fn prepare(request: &mut CodexRequest) -> Result<(), String> {
    if request
        .reasoning
        .as_ref()
        .is_none_or(|reasoning| reasoning.effort != "ultra")
    {
        return Ok(());
    }
    apply_ultra(request, &super::load_catalog().await?)
}

fn apply_ultra(request: &mut CodexRequest, models: &[CatalogModel]) -> Result<(), String> {
    let Some(reasoning) = request.reasoning.as_mut().filter(|r| r.effort == "ultra") else {
        return Ok(());
    };
    let model = models
        .iter()
        .find(|model| model.info.id == request.model)
        .ok_or_else(super::unavailable)?;
    let modes = &model.info.reasoning_modes;
    if !modes.iter().any(|mode| mode == "ultra") {
        return Err(super::unavailable());
    }
    // Unlike Codex's last-resort medium, fail closed if the catalogue has no
    // encodable effort. Do not send a mode that this model did not publish.
    reasoning.effort = model
        .multi_agent_reasoning_effort
        .as_ref()
        .or_else(|| modes.iter().find(|mode| mode.as_str() == "max"))
        .or_else(|| modes.iter().rev().find(|mode| mode.as_str() != "ultra"))
        .cloned()
        .ok_or_else(super::unavailable)?;
    Ok(())
}

pub(super) fn validated_modes(
    levels: Vec<crate::services::codex_client::model_catalog_wire::ReasoningLevel>,
) -> Vec<String> {
    let mut modes = Vec::with_capacity(levels.len());
    for level in levels {
        if super::ALLOWED_MODES.contains(&level.effort.as_str()) && !modes.contains(&level.effort) {
            modes.push(level.effort);
        }
    }
    modes
}

#[cfg(test)]
#[path = "model_catalog_reasoning_tests.rs"]
mod tests;
