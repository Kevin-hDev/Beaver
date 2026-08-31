use std::collections::BTreeSet;

use super::checkpoint_selection::CheckpointSelection;
use super::profile_types::CompressionBandSettings;
use super::snapshot::CompressionSnapshot;
use crate::services::agent_local::types_message::{AgentMessage, AgentMessageKind};

pub(super) fn prepare(
    snapshot: &CompressionSnapshot,
    selection: &CheckpointSelection,
    persisted: &[AgentMessage],
    band: &CompressionBandSettings,
) -> (
    Vec<super::checkpoint_attachments::CheckpointImage>,
    Vec<String>,
) {
    let source_ids: BTreeSet<String> = selection
        .messages
        .iter()
        .filter_map(|message| snapshot.source_messages.get(message.source_index()))
        .map(|message| message.id.clone())
        .collect();
    let images = super::checkpoint_attachments::retain_images_for_messages(
        &snapshot.checkpoint_images,
        &source_ids,
        usize::from(band.image_count),
        32 * 1024 * 1024,
    );
    let persisted_ids: BTreeSet<&str> = persisted
        .iter()
        .map(|message| message.id.as_str())
        .collect();
    let checkpoint_id = persisted
        .iter()
        .find(|message| message.message_kind == Some(AgentMessageKind::CompressionCheckpoint))
        .map(|message| message.id.clone());
    let images = images
        .into_iter()
        .map(|mut image| {
            if !persisted_ids.contains(image.source_message_id.as_str()) {
                if let Some(checkpoint_id) = &checkpoint_id {
                    image.source_message_id.clone_from(checkpoint_id);
                }
            }
            image
        })
        .collect();
    (images, source_ids.into_iter().collect())
}
