use serde::Serialize;

use super::checkpoint_document::CheckpointSection;
use crate::services::agent_local::types_message::AgentMessage;

#[derive(Serialize)]
struct RetainedMessage<'a> {
    source_message_id: &'a str,
    content: &'a str,
}

pub(super) fn append(
    sections: &mut Vec<CheckpointSection>,
    name: &'static str,
    messages: &[AgentMessage],
) -> Result<(), &'static str> {
    if messages.is_empty() {
        return Ok(());
    }
    let retained = messages
        .iter()
        .map(|message| RetainedMessage {
            source_message_id: &message.id,
            content: &message.content,
        })
        .collect::<Vec<_>>();
    sections.push(CheckpointSection {
        name: name.to_string(),
        content: serde_json::to_string(&retained).map_err(|_| "compression_candidate_invalid")?,
    });
    Ok(())
}
