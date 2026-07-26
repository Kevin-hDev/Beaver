use sha2::{Digest, Sha256};

const MAX_DELIVERED_PROMPT_HASHES: usize = 64;

pub struct ActiveSubagentRun {
    pub run_id: String,
    pub execution_id: String,
    pub cancelled: bool,
}

pub async fn get_run_id_for_child(child_id: &str) -> Option<String> {
    active_run_for_child(child_id)
        .await
        .map(|state| state.run_id)
}

pub async fn active_run_for_child(child_id: &str) -> Option<ActiveSubagentRun> {
    REGISTRY
        .lock()
        .await
        .entries
        .get(child_id)
        .map(|entry| ActiveSubagentRun {
            run_id: entry.run_id.clone(),
            execution_id: entry.execution_id.clone(),
            cancelled: entry.cancel.is_cancelled(),
        })
}

pub async fn owns_execution(child_id: &str, run_id: &str, execution_id: &str) -> bool {
    REGISTRY
        .lock()
        .await
        .entries
        .get(child_id)
        .is_some_and(|entry| entry.run_id == run_id && entry.execution_id == execution_id)
}

pub async fn prompt_was_delivered(child_id: &str, execution_id: &str, prompt: &str) -> bool {
    let hash = prompt_hash(prompt);
    REGISTRY
        .lock()
        .await
        .entries
        .get(child_id)
        .filter(|entry| entry.execution_id == execution_id)
        .is_some_and(|entry| {
            entry.initial_prompt_hash == Some(hash)
                || entry.delivered_prompt_hashes.contains(&hash)
        })
}

pub async fn save_queued_prompt_for_execution(
    child: &super::types_session::AgentSession,
    execution_id: &str,
) -> Result<(), String> {
    let state = REGISTRY.lock().await;
    let entry = state
        .entries
        .get(&child.id)
        .filter(|entry| {
            entry.execution_id == execution_id
                && !entry.cancel.is_cancelled()
                && child.subagent_run_id.as_deref() == Some(&entry.run_id)
        })
        .ok_or_else(|| "Livraison sous-agent obsolète".to_string())?;
    if entry
        .delivered_prompt_hashes
        .len()
        .saturating_add(child.subagent_queued_prompts.len())
        > MAX_DELIVERED_PROMPT_HASHES
    {
        return Err("Limite de corrections sous-agent atteinte".to_string());
    }
    super::session_store::save(child).await
}

pub async fn save_and_mark_prompts_delivered(
    child: &super::types_session::AgentSession,
    execution_id: &str,
    prompts: &[String],
) -> Result<(), String> {
    let hashes = unique_new_hashes(prompts);
    let mut state = REGISTRY.lock().await;
    let entry = state
        .entries
        .get_mut(&child.id)
        .filter(|entry| {
            entry.execution_id == execution_id
                && !entry.cancel.is_cancelled()
                && child.subagent_run_id.as_deref() == Some(&entry.run_id)
        })
        .ok_or_else(|| "Livraison sous-agent obsolète".to_string())?;
    let new_hashes = hashes
        .into_iter()
        .filter(|hash| !entry.delivered_prompt_hashes.contains(hash))
        .collect::<Vec<_>>();
    if entry.delivered_prompt_hashes.len().saturating_add(new_hashes.len())
        > MAX_DELIVERED_PROMPT_HASHES
    {
        return Err("Limite de corrections sous-agent atteinte".to_string());
    }
    super::session_store::save(child).await?;
    entry.delivered_prompt_hashes.extend(new_hashes);
    Ok(())
}

fn unique_new_hashes(prompts: &[String]) -> Vec<[u8; 32]> {
    let mut hashes = Vec::with_capacity(prompts.len());
    for hash in prompts.iter().map(|prompt| prompt_hash(prompt)) {
        if !hashes.contains(&hash) {
            hashes.push(hash);
        }
    }
    hashes
}

fn prompt_hash(prompt: &str) -> [u8; 32] {
    let normalized = super::subagent_instruction_delivery::normalized_prompt(prompt);
    Sha256::digest(normalized.as_bytes()).into()
}

pub async fn cancel_execution(child_id: &str, expected_execution_id: &str) -> bool {
    let state = REGISTRY.lock().await;
    let Some(entry) = state
        .entries
        .get(child_id)
        .filter(|entry| entry.execution_id == expected_execution_id)
    else {
        return false;
    };
    entry.cancel.cancel();
    true
}

pub async fn adopt_children_for_parent_stream(
    parent_id: &str,
    parent_cancel: &CancellationToken,
) {
    if parent_cancel.is_cancelled() {
        return;
    }
    let mut state = REGISTRY.lock().await;
    for entry in state.entries.values_mut() {
        if entry.parent_session_id == parent_id {
            entry.parent_stream_cancel = parent_cancel.clone();
        }
    }
}

pub async fn cancel_stopped_parent_stream_children(parent_id: &str) {
    let state = REGISTRY.lock().await;
    for entry in state.entries.values() {
        if entry.parent_session_id == parent_id && entry.parent_stream_cancel.is_cancelled() {
            entry.cancel.cancel();
        }
    }
}

#[cfg(test)]
pub async fn cancel_one(child_id: &str) -> bool {
    let state = REGISTRY.lock().await;
    if let Some(entry) = state.entries.get(child_id) {
        entry.cancel.cancel();
        true
    } else {
        false
    }
}

#[cfg(test)]
pub async fn cancel_all_for_parent(parent_id: &str) {
    let state = REGISTRY.lock().await;
    for entry in state.entries.values() {
        if entry.parent_session_id == parent_id {
            entry.cancel.cancel();
        }
    }
}
