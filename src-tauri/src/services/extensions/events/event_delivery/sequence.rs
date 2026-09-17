use super::EventDraft;
use std::collections::BTreeMap;

pub(super) fn next(sequences: &mut BTreeMap<String, u64>, draft: &EventDraft) -> Option<u64> {
    if draft.starts_flow {
        if sequences.contains_key(&draft.flow_id)
            || sequences.len() >= super::super::types::MAX_ACTIVE_CONTEXTS
        {
            return None;
        }
        sequences.insert(draft.flow_id.clone(), 1);
        return Some(1);
    }
    let sequence = sequences.get_mut(&draft.flow_id)?;
    *sequence = sequence.saturating_add(1);
    let value = *sequence;
    if draft.terminal {
        sequences.remove(&draft.flow_id);
    }
    Some(value)
}
