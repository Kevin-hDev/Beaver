use std::ops::Range;

use crate::services::agent_local::types_session::AgentMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointUnitKind {
    ActiveTurn,
    UserMessage,
    AssistantMessage,
    ToolChain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointUnit {
    pub message_indexes: Range<usize>,
    pub estimated_tokens: u32,
    pub kind: CheckpointUnitKind,
    pub required: bool,
}

pub fn build(messages: &[AgentMessage]) -> Result<Vec<CheckpointUnit>, &'static str> {
    let turns = crate::services::agent_local::conversation_compaction::turn_ranges(messages);
    let mut units = Vec::new();
    for (turn_index, range) in turns.iter().enumerate() {
        let turn = &messages[range.clone()];
        let terminal =
            crate::services::agent_local::conversation_compaction::is_terminal_turn(turn);
        let active = turn_index + 1 == turns.len() && !terminal;
        if active {
            super::checkpoint_tools::validate_active_turn(turn)?;
            units.push(unit(
                messages,
                range.clone(),
                CheckpointUnitKind::ActiveTurn,
                true,
            ));
            continue;
        }
        if !terminal {
            return Err("compression_checkpoint_invalid");
        }
        append_complete_turn_units(messages, range.clone(), &mut units)?;
    }
    Ok(units)
}

fn append_complete_turn_units(
    messages: &[AgentMessage],
    range: Range<usize>,
    units: &mut Vec<CheckpointUnit>,
) -> Result<(), &'static str> {
    if messages
        .get(range.start)
        .is_none_or(|message| message.role != "user")
    {
        return Err("compression_checkpoint_invalid");
    }
    units.push(unit(
        messages,
        range.start..range.start + 1,
        CheckpointUnitKind::UserMessage,
        false,
    ));
    let mut index = range.start + 1;
    while index < range.end {
        let message = &messages[index];
        match message.role.as_str() {
            "assistant"
                if message
                    .tool_calls
                    .as_ref()
                    .is_some_and(|calls| !calls.is_empty()) =>
            {
                let end = super::checkpoint_tools::closed_chain_end(messages, index, range.end)?;
                units.push(unit(
                    messages,
                    index..end,
                    CheckpointUnitKind::ToolChain,
                    false,
                ));
                index = end;
            }
            "assistant" => {
                if index + 1 != range.end {
                    return Err("compression_checkpoint_invalid");
                }
                units.push(unit(
                    messages,
                    index..index + 1,
                    CheckpointUnitKind::AssistantMessage,
                    false,
                ));
                index += 1;
            }
            _ => return Err("compression_checkpoint_invalid"),
        }
    }
    Ok(())
}

fn unit(
    messages: &[AgentMessage],
    range: Range<usize>,
    kind: CheckpointUnitKind,
    required: bool,
) -> CheckpointUnit {
    let estimated_tokens = messages[range.clone()].iter().fold(0u32, |total, message| {
        total.saturating_add(super::token_estimate::estimate_checkpoint_message_tokens(
            message,
        ))
    });
    CheckpointUnit {
        message_indexes: range,
        estimated_tokens,
        kind,
        required,
    }
}
