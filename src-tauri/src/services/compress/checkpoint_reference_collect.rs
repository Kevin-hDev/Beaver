use super::checkpoint_evidence::EvidenceItemLimit;
use super::checkpoint_references::CheckpointReference;

pub fn collect(
    messages: &[crate::services::agent_local::types_message::AgentMessage],
    budget: EvidenceItemLimit,
) -> Vec<CheckpointReference> {
    let mut candidates = Vec::new();
    for message in messages {
        candidates.extend(message.files.iter().map(|file| CheckpointReference {
            kind: "attachment".into(),
            id: if file.path.is_empty() {
                file.name.clone()
            } else {
                file.path.clone()
            },
            label: file.name.clone(),
        }));
        if let Some(calls) = &message.tool_calls {
            candidates.extend(calls.iter().map(|call| CheckpointReference {
                kind: "tool_call".into(),
                id: call.id.clone(),
                label: call.function.name.clone(),
            }));
        }
        if let Some(ids) = &message.skill_ids {
            candidates.extend(ids.iter().enumerate().map(|(index, id)| {
                CheckpointReference {
                    kind: "skill".into(),
                    id: id.clone(),
                    label: message
                        .skill_names
                        .as_ref()
                        .and_then(|names| names.get(index))
                        .cloned()
                        .unwrap_or_else(|| id.clone()),
                }
            }));
        }
    }
    super::checkpoint_references::collect(
        candidates,
        usize::from(budget.max_items),
        budget.total_tokens,
    )
}
