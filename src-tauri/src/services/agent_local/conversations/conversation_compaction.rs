use super::types_message::AgentMessage;

pub(crate) fn turn_ranges(messages: &[AgentMessage]) -> Vec<std::ops::Range<usize>> {
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

pub(crate) fn is_terminal_turn(turn: &[AgentMessage]) -> bool {
    turn.last()
        .is_some_and(|message| message.role == "assistant" && message.tool_calls.is_none())
}

/// Renvoie une frontière exclusive qui ne coupe jamais un tour. Un clone
/// demandé sur un message utilisateur repart donc du dernier tour terminé.
pub(super) fn terminal_prefix_end(messages: &[AgentMessage], message_index: usize) -> usize {
    turn_ranges(messages)
        .into_iter()
        .filter(|range| {
            range.end.saturating_sub(1) <= message_index
                && is_terminal_turn(&messages[range.clone()])
        })
        .map(|range| range.end)
        .max()
        .unwrap_or(0)
}
