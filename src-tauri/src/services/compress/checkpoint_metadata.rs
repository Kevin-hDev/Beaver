use serde::Serialize;

use super::checkpoint_document::CheckpointSection;
use super::profile_types::CompressionTrigger;
use crate::services::agent_local::types_message::{AgentMessage, AgentMessageKind};

#[derive(Serialize)]
struct CheckpointMetadata<'a> {
    profile_id: &'a str,
    profile_revision: u64,
    before_tokens: u32,
    after_tokens: u32,
    trigger: CompressionTrigger,
    retained_message_ids: Vec<String>,
    section_names: Vec<String>,
}

pub(super) fn set(
    messages: &mut [AgentMessage],
    snapshot: &super::snapshot::CompressionSnapshot,
    after_tokens: u32,
    sections: &[CheckpointSection],
    retained_message_ids: Vec<String>,
) -> Result<(), &'static str> {
    let metadata = CheckpointMetadata {
        profile_id: &snapshot.profile.profile.id,
        profile_revision: snapshot.profile.profile_revision,
        before_tokens: snapshot.before_tokens,
        after_tokens,
        trigger: snapshot.trigger,
        retained_message_ids,
        section_names: sections
            .iter()
            .map(|section| section.name.clone())
            .collect(),
    };
    let checkpoint = messages
        .iter_mut()
        .find(|message| message.message_kind == Some(AgentMessageKind::CompressionCheckpoint))
        .ok_or("compression_candidate_invalid")?;
    let mut body: serde_json::Value =
        serde_json::from_str(&checkpoint.content).map_err(|_| "compression_candidate_invalid")?;
    body.as_object_mut()
        .ok_or("compression_candidate_invalid")?
        .insert(
            "metadata".into(),
            serde_json::to_value(metadata).map_err(|_| "compression_candidate_invalid")?,
        );
    checkpoint.content =
        serde_json::to_string_pretty(&body).map_err(|_| "compression_candidate_invalid")?;
    Ok(())
}
