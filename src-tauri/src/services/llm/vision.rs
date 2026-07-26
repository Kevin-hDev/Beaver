use crate::services::agent_local::types_ollama::ChatMessage;
use serde_json::{json, Value};

pub const MAX_IMAGES_PER_MESSAGE: usize = 8;
pub const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
pub const IMAGE_TOKEN_ESTIMATE: usize = 1_100;

pub const NOTICE_UNSUPPORTED_MODEL: &str = "vision.unsupportedModel";
pub const NOTICE_IMAGE_SKIPPED: &str = "vision.imageSkipped";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImageSanitizeReport {
    pub unsupported_removed: usize,
    pub invalid_removed: usize,
}

pub fn sanitize_messages(
    messages: &mut [ChatMessage],
    supports_vision: bool,
) -> ImageSanitizeReport {
    let mut report = ImageSanitizeReport::default();
    for msg in messages.iter_mut().filter(|m| m.role == "user") {
        let Some(images) = msg.images.take() else {
            continue;
        };
        if images.is_empty() {
            continue;
        }
        if !supports_vision {
            report.unsupported_removed += images.len();
            continue;
        }
        let original_len = images.len();
        let mut kept = Vec::new();
        for image in images.into_iter().take(MAX_IMAGES_PER_MESSAGE) {
            if image_payload(&image).is_some() {
                kept.push(normalize_base64(&image));
            } else {
                report.invalid_removed += 1;
            }
        }
        report.invalid_removed += original_len.saturating_sub(MAX_IMAGES_PER_MESSAGE);
        msg.images = if kept.is_empty() { None } else { Some(kept) };
    }
    report
}

pub fn strip_images(messages: &mut [ChatMessage]) -> usize {
    let mut removed = 0;
    for msg in messages.iter_mut().filter(|m| m.role == "user") {
        if let Some(images) = msg.images.take() {
            let count = images.len();
            removed += count;
            if count > 0 {
                let note = if count == 1 {
                    "\n\n[1 image was attached but this model does not support vision]"
                } else {
                    "\n\n[Images were attached but this model does not support vision]"
                };
                msg.content.push_str(note);
            }
        }
    }
    removed
}

pub fn openai_image_part(base64_data: &str, provider_id: &str) -> Value {
    let data_url = data_url(base64_data);
    match provider_id {
        "mistral" => json!({ "type": "image_url", "image_url": data_url }),
        _ => json!({ "type": "image_url", "image_url": { "url": data_url } }),
    }
}

pub fn responses_image_part(base64_data: &str) -> Value {
    json!({ "type": "input_image", "image_url": data_url(base64_data) })
}

pub fn data_url(base64_data: &str) -> String {
    let normalized = normalize_base64(base64_data);
    let mime = detect_mime(&normalized);
    format!("data:{mime};base64,{normalized}")
}

pub fn detect_mime(b64: &str) -> &'static str {
    let normalized = normalize_base64(b64);
    let prefix = &normalized[..normalized.len().min(16)];
    if prefix.starts_with("/9j/") {
        "image/jpeg"
    } else if prefix.starts_with("iVBOR") {
        "image/png"
    } else if prefix.starts_with("R0lGO") {
        "image/gif"
    } else if prefix.starts_with("UklGR") {
        "image/webp"
    } else {
        "image/png"
    }
}

fn image_payload(input: &str) -> Option<&str> {
    let payload = input
        .split_once("base64,")
        .map_or(input, |(_, data)| data)
        .trim();
    if payload.is_empty() || decoded_len_estimate(payload) > MAX_IMAGE_BYTES {
        return None;
    }
    if has_supported_image_signature(payload) {
        Some(payload)
    } else {
        None
    }
}

fn has_supported_image_signature(payload: &str) -> bool {
    payload.starts_with("/9j/")
        || payload.starts_with("iVBOR")
        || payload.starts_with("R0lGO")
        || payload.starts_with("UklGR")
}

fn normalize_base64(input: &str) -> String {
    input
        .split_once("base64,")
        .map_or(input, |(_, data)| data)
        .trim()
        .to_string()
}

fn decoded_len_estimate(b64: &str) -> usize {
    b64.len().saturating_mul(3) / 4
}

#[cfg(test)]
#[path = "vision_tests.rs"]
mod tests;
