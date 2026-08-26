use crate::services::reasoning_continuity::envelope::ReasoningEnvelope;

use super::types_message::AgentMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionError {
    OpenTurn,
}

#[derive(Debug, Clone)]
pub struct CompactionOutcome {
    pub removed_turns: usize,
    pub replaced_envelopes: Vec<ReasoningEnvelope>,
}

/// Drops only terminal turns. A pending tool chain is intentionally a hard stop:
/// a summary cannot faithfully replace an unfinished provider interaction.
pub fn compact_complete_turns(
    messages: &mut Vec<AgentMessage>,
    keep_complete_turns: usize,
) -> Result<CompactionOutcome, CompactionError> {
    let ranges = turn_ranges(messages);
    if ranges.iter().any(|range| is_open_tool_chain(&messages[range.clone()])) {
        return Err(CompactionError::OpenTurn);
    }
    let complete = ranges
        .iter()
        .filter(|range| is_terminal_turn(&messages[range.start..range.end]))
        .cloned()
        .collect::<Vec<_>>();
    let remove_count = complete.len().saturating_sub(keep_complete_turns);
    let remove_end = complete.get(remove_count.saturating_sub(1)).map(|range| range.end);
    let Some(remove_end) = remove_end else {
        return Ok(CompactionOutcome { removed_turns: 0, replaced_envelopes: Vec::new() });
    };
    let mut replaced_envelopes = messages[..remove_end]
        .iter_mut()
        .filter_map(|message| message.continuation.as_mut())
        .map(|envelope| {
            envelope.completion = crate::services::reasoning_continuity::envelope::CompletionState::Compacted;
            envelope.clone()
        })
        .collect::<Vec<_>>();
    replaced_envelopes.shrink_to_fit();
    messages.drain(..remove_end);
    Ok(CompactionOutcome { removed_turns: remove_count, replaced_envelopes })
}

fn turn_ranges(messages: &[AgentMessage]) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    while start < messages.len() {
        let end = messages[start + 1..]
            .iter()
            .position(|message| message.turn_id != messages[start].turn_id)
            .map_or(messages.len(), |offset| start + offset + 1);
        ranges.push(start..end);
        start = end;
    }
    ranges
}

fn is_open_tool_chain(turn: &[AgentMessage]) -> bool {
    turn.iter().any(|message| {
        message.role == "assistant"
            && message.tool_calls.as_ref().is_some_and(|calls| !calls.is_empty())
            && !turn.iter().any(|candidate| candidate.role == "tool")
    })
}

fn is_terminal_turn(turn: &[AgentMessage]) -> bool {
    turn.last().is_some_and(|message| message.role == "assistant" && message.tool_calls.is_none())
}
