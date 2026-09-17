use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::models::agent_turn_contract::TurnAttachmentInput;
use crate::services::attachment_access::MAX_ATTACHMENT_SIZE;

use super::conversation_attachment_format::{self, AttachmentPayload};
use super::conversation_input::{
    ConversationInputError, ConversationInputErrorKind, MAX_TEXT_CHARS_PER_FILE,
};
use super::types_message::FileAttachment;

const MAX_TEXT_BYTES_PER_FILE: u64 = (MAX_TEXT_CHARS_PER_FILE as u64) * 4;

#[derive(Debug)]
pub struct ResolvedAttachment {
    pub file: FileAttachment,
    pub content: ResolvedAttachmentContent,
    identity: AttachmentIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum AttachmentIdentity {
    File(String),
    Inline { name: String, digest: [u8; 32] },
}

#[derive(Debug)]
pub enum ResolvedAttachmentContent {
    Text(String),
    Image(ResolvedImage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImage {
    pub mime_type: String,
    pub base64: String,
}

impl ResolvedAttachment {
    pub(super) fn identity(&self) -> AttachmentIdentity {
        self.identity.clone()
    }
}

pub async fn resolve(
    input: TurnAttachmentInput,
    key: &[u8],
) -> Result<ResolvedAttachment, ConversationInputError> {
    conversation_attachment_format::validate_wire_bounds(&input)?;
    conversation_attachment_format::validate_name(&input.name)?;
    if input.path.is_empty() {
        return resolve_inline(input);
    }
    let grant = input
        .access_grant
        .as_deref()
        .ok_or_else(|| error(ConversationInputErrorKind::Grant))?;
    let max_bytes = if conversation_attachment_format::is_text_name(&input.name) {
        MAX_TEXT_BYTES_PER_FILE
    } else {
        MAX_ATTACHMENT_SIZE
    };
    let raw = input.path.clone();
    let grant = grant.to_string();
    let key = Zeroizing::new(key.to_vec());
    let verified = tokio::task::spawn_blocking(move || {
        crate::services::attachment_access::read_verified(&raw, &grant, &key, max_bytes)
    })
    .await
    .map_err(|_| error(ConversationInputErrorKind::Unavailable))?
    .map_err(|failure| match failure {
        crate::services::attachment_access::VerifiedAttachmentError::Access => {
            error(ConversationInputErrorKind::Grant)
        }
        crate::services::attachment_access::VerifiedAttachmentError::Limit => {
            error(ConversationInputErrorKind::Limit)
        }
        crate::services::attachment_access::VerifiedAttachmentError::Unavailable => {
            error(ConversationInputErrorKind::Unavailable)
        }
    })?;
    if verified
        .canonical_path
        .file_name()
        .and_then(|value| value.to_str())
        != Some(input.name.as_str())
        || (input.size != 0 && input.size != verified.bytes.len() as u64)
    {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    let canonical = verified
        .canonical_path
        .to_str()
        .ok_or_else(|| error(ConversationInputErrorKind::Invalid))?
        .to_string();
    build_resolved(input, canonical, verified.bytes)
}

fn resolve_inline(
    input: TurnAttachmentInput,
) -> Result<ResolvedAttachment, ConversationInputError> {
    if input.access_grant.is_some()
        || conversation_attachment_format::is_text_name(&input.name)
        || input.size > MAX_ATTACHMENT_SIZE
    {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    let thumbnail = input
        .thumbnail
        .as_deref()
        .ok_or_else(|| error(ConversationInputErrorKind::Invalid))?;
    let decoded = conversation_attachment_format::decode_data_url(thumbnail)?;
    if !input.mime_type.eq_ignore_ascii_case(&decoded.mime_type) {
        return Err(error(ConversationInputErrorKind::Type));
    }
    let bytes = decoded.bytes;
    if input.size != 0 && input.size != bytes.len() as u64 {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    build_resolved(input, String::new(), bytes)
}

fn build_resolved(
    input: TurnAttachmentInput,
    path: String,
    bytes: Vec<u8>,
) -> Result<ResolvedAttachment, ConversationInputError> {
    let identity = if path.is_empty() {
        AttachmentIdentity::Inline {
            name: input.name.clone(),
            digest: Sha256::digest(&bytes).into(),
        }
    } else {
        AttachmentIdentity::File(path.clone())
    };
    let payload = conversation_attachment_format::resolve(&input.name, &input.mime_type, bytes)?;
    let (content, thumbnail, mime_type, size) = match payload {
        AttachmentPayload::Text(text) => {
            let size = text.len() as u64;
            (
                ResolvedAttachmentContent::Text(text),
                None,
                "text/plain".to_string(),
                size,
            )
        }
        AttachmentPayload::Image(image) => {
            let size = image.byte_len;
            let thumbnail = Some(format!("data:{};base64,{}", image.mime_type, image.base64));
            let resolved = ResolvedImage {
                mime_type: image.mime_type.clone(),
                base64: image.base64,
            };
            (
                ResolvedAttachmentContent::Image(resolved),
                thumbnail,
                image.mime_type,
                size,
            )
        }
    };
    Ok(ResolvedAttachment {
        file: FileAttachment {
            name: input.name,
            path,
            mime_type,
            size,
            thumbnail,
            access_grant: input.access_grant,
        },
        content,
        identity,
    })
}

fn error(kind: ConversationInputErrorKind) -> ConversationInputError {
    ConversationInputError::new(kind)
}
