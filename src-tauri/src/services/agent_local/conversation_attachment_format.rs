use base64::{engine::general_purpose::STANDARD, Engine};

use crate::models::agent_turn_contract::{
    TurnAttachmentInput, MAX_ATTACHMENT_GRANT_BYTES, MAX_ATTACHMENT_MIME_BYTES,
    MAX_ATTACHMENT_NAME_BYTES, MAX_ATTACHMENT_PATH_BYTES, MAX_ATTACHMENT_THUMBNAIL_BYTES,
};
use crate::services::attachment_access::MAX_ATTACHMENT_SIZE;

use super::conversation_input::{
    ConversationInputError, ConversationInputErrorKind, MAX_TEXT_CHARS_PER_FILE,
};

const MAX_NAME_CHARS: usize = 255;
pub(super) enum AttachmentPayload {
    Text(String),
    Image(ResolvedImagePayload),
}

pub(super) struct ResolvedImagePayload {
    pub mime_type: String,
    pub base64: String,
    pub byte_len: u64,
}

pub(super) struct DecodedDataUrl {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

pub(super) fn validate_wire_bounds(
    input: &TurnAttachmentInput,
) -> Result<(), ConversationInputError> {
    validate_fields(
        &input.name,
        &input.path,
        &input.mime_type,
        input.size,
        input.thumbnail.as_deref(),
        input.access_grant.as_deref(),
    )
}

pub(super) fn validate_persisted(
    file: &super::types_message::FileAttachment,
) -> Result<(), ConversationInputError> {
    validate_fields(
        &file.name,
        &file.path,
        &file.mime_type,
        file.size,
        file.thumbnail.as_deref(),
        file.access_grant.as_deref(),
    )?;
    validate_name(&file.name)
}

fn validate_fields(
    name: &str,
    path: &str,
    mime_type: &str,
    size: u64,
    thumbnail: Option<&str>,
    access_grant: Option<&str>,
) -> Result<(), ConversationInputError> {
    if name.len() > MAX_ATTACHMENT_NAME_BYTES
        || path.len() > MAX_ATTACHMENT_PATH_BYTES
        || mime_type.len() > MAX_ATTACHMENT_MIME_BYTES
        || thumbnail.is_some_and(|value| value.len() > MAX_ATTACHMENT_THUMBNAIL_BYTES)
        || access_grant.is_some_and(|value| value.len() > MAX_ATTACHMENT_GRANT_BYTES)
        || size > MAX_ATTACHMENT_SIZE
    {
        return Err(error(ConversationInputErrorKind::Limit));
    }
    if mime_type.is_empty() {
        return Err(error(ConversationInputErrorKind::Type));
    }
    if [name, path, mime_type]
        .into_iter()
        .any(|value| value.chars().any(char::is_control))
        || thumbnail.is_some_and(|value| value.chars().any(char::is_control))
        || access_grant.is_some_and(|value| value.chars().any(char::is_control))
    {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    Ok(())
}

pub(super) fn validate_name(name: &str) -> Result<(), ConversationInputError> {
    if name.is_empty()
        || name.chars().count() > MAX_NAME_CHARS
        || name.chars().any(char::is_control)
        || name.contains(['/', '\\'])
        || matches!(name, "." | "..")
    {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    Ok(())
}

pub(super) fn is_text_name(name: &str) -> bool {
    super::conversation_attachment_types::is_text_name(name)
}

pub(super) fn resolve(
    name: &str,
    declared: &str,
    bytes: Vec<u8>,
) -> Result<AttachmentPayload, ConversationInputError> {
    let extension = super::conversation_attachment_types::extension(name)
        .ok_or_else(|| error(ConversationInputErrorKind::Type))?;
    if super::conversation_attachment_types::is_text_extension(&extension) {
        if !super::conversation_attachment_types::declared_text_type(&extension, declared) {
            return Err(error(ConversationInputErrorKind::Type));
        }
        let text = String::from_utf8(bytes).map_err(|_| error(ConversationInputErrorKind::Type))?;
        if text.contains('\0') || text.chars().count() > MAX_TEXT_CHARS_PER_FILE {
            return Err(error(ConversationInputErrorKind::Limit));
        }
        return Ok(AttachmentPayload::Text(text));
    }
    let format = crate::services::file_signature::classify(&bytes);
    let normalized_extension = match extension.as_str() {
        "jpg" | "jpeg" => "jpeg",
        "png" => "png",
        "gif" => "gif",
        "webp" => "webp",
        _ => return Err(error(ConversationInputErrorKind::Type)),
    };
    if !format.matches_image_extension(normalized_extension)
        || !format.matches_declared_image(declared)
    {
        return Err(error(ConversationInputErrorKind::Type));
    }
    Ok(AttachmentPayload::Image(ResolvedImagePayload {
        mime_type: format.mime().to_string(),
        base64: STANDARD.encode(&bytes),
        byte_len: bytes.len() as u64,
    }))
}

pub(super) fn decode_data_url(value: &str) -> Result<DecodedDataUrl, ConversationInputError> {
    let (header, payload) = value
        .trim()
        .split_once(',')
        .ok_or_else(|| error(ConversationInputErrorKind::Type))?;
    let mime_type = header
        .strip_prefix("data:")
        .and_then(|value| value.strip_suffix(";base64"))
        .filter(|value| value.starts_with("image/") && value.len() <= 64)
        .ok_or_else(|| error(ConversationInputErrorKind::Type))?;
    if mime_type.chars().any(char::is_control) {
        return Err(error(ConversationInputErrorKind::Type));
    }
    let max_encoded = ((MAX_ATTACHMENT_SIZE as usize).div_ceil(3)) * 4;
    if payload.is_empty() || payload.len() > max_encoded {
        return Err(error(ConversationInputErrorKind::Limit));
    }
    let bytes = STANDARD
        .decode(payload.as_bytes())
        .map_err(|_| error(ConversationInputErrorKind::Type))?;
    if bytes.len() as u64 > MAX_ATTACHMENT_SIZE {
        return Err(error(ConversationInputErrorKind::Limit));
    }
    Ok(DecodedDataUrl {
        bytes,
        mime_type: mime_type.to_string(),
    })
}

fn error(kind: ConversationInputErrorKind) -> ConversationInputError {
    ConversationInputError::new(kind)
}
