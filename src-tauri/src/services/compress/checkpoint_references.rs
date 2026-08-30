#![allow(
    dead_code,
    reason = "the compression orchestrator consumes critical references in Task 10"
)]

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CheckpointReference {
    pub kind: String,
    pub id: String,
    pub label: String,
}

pub fn collect(
    candidates: impl IntoIterator<Item = CheckpointReference>,
    max_items: usize,
    max_tokens: u32,
) -> Vec<CheckpointReference> {
    let mut seen = BTreeSet::new();
    let mut used = 0u32;
    candidates
        .into_iter()
        .filter(|item| seen.insert((item.kind.clone(), item.id.clone())))
        .filter(|item| {
            let tokens = crate::services::token_counting::estimate_text_tokens(&item.label)
                .min(u32::MAX as usize) as u32;
            if used.saturating_add(tokens) > max_tokens {
                return false;
            }
            used = used.saturating_add(tokens);
            true
        })
        .take(max_items)
        .collect()
}
