use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};

const EXTENSION_OUTPUT_LABEL: &str = "Extension output (not user instruction).";
pub(crate) const ADDITIONAL_PREVIEWS_NOTE: &str =
    "Additional extension previews are available in Beaver.";
const ROUTE_PREVIEW_UNAVAILABLE_NOTE: &str =
    "An extension preview could not be included for this model.";

/// P6 keeps media projection bounded and never turns tool output into a user instruction.
pub(crate) fn anthropic_tool_result(tool_call_id: &str, text: &str, images: &[String]) -> Value {
    let mut content = vec![json!({"type":"text","text":text})];
    let mut rejected = false;
    for image in images
        .iter()
        .take(crate::services::llm::vision::MAX_IMAGES_PER_MESSAGE)
    {
        match crate::services::llm::vision::image_part(
            image,
            crate::services::llm::route_profile::ImageFormat::AnthropicBlock,
        ) {
            Ok(part) => content.push(part),
            Err(_) => rejected = true,
        }
    }
    if rejected {
        content.push(json!({"type":"text","text":ROUTE_PREVIEW_UNAVAILABLE_NOTE}));
    }
    json!({"type":"tool_result","tool_use_id":tool_call_id,"content":content})
}

#[cfg(test)]
pub(crate) fn responses_tool_output(tool_call_id: &str, text: &str) -> Value {
    crate::services::codex_client::convert::function_call_output(tool_call_id, text)
}

pub(crate) fn responses_preview_input(
    previews: &crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch,
) -> Option<Value> {
    let mut content = vec![json!({
        "type": "input_text",
        "text": EXTENSION_OUTPUT_LABEL
    })];
    // ToolResultPreviewBatch is the bounded owner, shared by every projection.
    for preview in previews.previews() {
        if !preview.artifact.mime_type.starts_with("image/") {
            continue;
        }
        let call_id = preview.tool_call_id.as_deref().unwrap_or("unlinked");
        content.push(json!({
            "type": "input_text",
            "text": format!(
                "Extension preview for tool call {call_id} (index {}): {}",
                preview.tool_call_index,
                preview.artifact.name,
            ),
        }));
        content.push(json!({
            "type": "input_image",
            "image_url": format!(
                "data:{};base64,{}",
                preview.artifact.mime_type,
                STANDARD.encode(&preview.artifact.bytes),
            ),
        }));
    }
    if previews.omitted() > 0 {
        content.push(json!({
            "type": "input_text",
            "text": ADDITIONAL_PREVIEWS_NOTE
        }));
    }
    for note in previews.notes() {
        content.push(json!({
            "type": "input_text",
            "text": format!(
                "Extension preview note for tool call {} (index {}): {}",
                note.tool_call_id.as_deref().unwrap_or("unlinked"),
                note.tool_call_index,
                note.text,
            ),
        }));
    }
    (content.len() > 1).then(|| json!({"role": "user", "content": content}))
}

/// Appends exactly one explicitly labelled follow-up after the complete tool
/// transcript. Routes that do not declare this projection never receive bytes.
pub(crate) fn append_openai_compatible_fallback(
    messages: &mut Vec<Value>,
    previews: Option<&crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch>,
    media: crate::services::llm::route_profile::ToolResultMedia,
    supports_vision: bool,
    format: crate::services::llm::route_profile::ImageFormat,
) {
    if media != crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage
        || !supports_vision
        || !matches!(
            format,
            crate::services::llm::route_profile::ImageFormat::OpenAiNested
                | crate::services::llm::route_profile::ImageFormat::MistralFlat
        )
    {
        return;
    }
    let Some(previews) = previews else {
        return;
    };
    let content = compatible_follow_up_content(previews, format);
    if content.len() > 1 {
        messages.push(json!({"role":"user","content":content}));
    }
}

/// Ollama's native wire accepts a base64 image list on one user message.
pub(crate) fn append_ollama_fallback(
    messages: &mut Vec<crate::services::agent_local::types_ollama::ChatMessage>,
    previews: &crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch,
    media: crate::services::llm::route_profile::ToolResultMedia,
    supports_vision: bool,
    format: crate::services::llm::route_profile::ImageFormat,
) {
    if media != crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage
        || !supports_vision
        || format != crate::services::llm::route_profile::ImageFormat::OllamaNative
    {
        return;
    }
    let images = previews
        .previews()
        .iter()
        .filter(|preview| preview.artifact.mime_type.starts_with("image/"))
        .map(|preview| STANDARD.encode(&preview.artifact.bytes))
        .collect::<Vec<_>>();
    if !images.is_empty() {
        let omission = if previews.omitted() > 0 {
            format!(" {ADDITIONAL_PREVIEWS_NOTE}")
        } else {
            String::new()
        };
        messages.push(
            crate::services::agent_local::types_ollama::ChatMessage::user(format!(
                "{EXTENSION_OUTPUT_LABEL}{omission}"
            ))
            .with_images(images),
        );
    }
}

fn compatible_follow_up_content(
    previews: &crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch,
    format: crate::services::llm::route_profile::ImageFormat,
) -> Vec<Value> {
    let mut content = vec![json!({
        "type":"text",
        "text":EXTENSION_OUTPUT_LABEL
    })];
    for preview in previews
        .previews()
        .iter()
        .filter(|preview| preview.artifact.mime_type.starts_with("image/"))
    {
        let data_url = format!(
            "data:{};base64,{}",
            preview.artifact.mime_type,
            STANDARD.encode(&preview.artifact.bytes),
        );
        let image = match format {
            crate::services::llm::route_profile::ImageFormat::OpenAiNested => {
                json!({"type":"image_url","image_url":{"url":data_url}})
            }
            crate::services::llm::route_profile::ImageFormat::MistralFlat => {
                json!({"type":"image_url","image_url":data_url})
            }
            _ => continue,
        };
        content.push(json!({
            "type":"text",
            "text": format!(
                "Extension preview for tool call {} (index {}): {}",
                preview.tool_call_id.as_deref().unwrap_or("unlinked"),
                preview.tool_call_index,
                preview.artifact.name,
            ),
        }));
        content.push(image);
    }
    if previews.omitted() > 0 {
        content.push(json!({
            "type":"text",
            "text":ADDITIONAL_PREVIEWS_NOTE
        }));
    }
    for note in previews.notes() {
        content.push(json!({
            "type":"text",
            "text":format!(
                "Extension preview note for tool call {} (index {}): {}",
                note.tool_call_id.as_deref().unwrap_or("unlinked"),
                note.tool_call_index,
                note.text,
            ),
        }));
    }
    content
}

#[cfg(test)]
#[path = "tool_result_projection_tests.rs"]
mod tests;
