use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub use super::subagent_terminal_signal::SubagentTerminalKind;
use super::subagent_terminal_signal::{SubagentTerminalNotifier, SubagentTerminalState};

const MAX_PER_PARENT: usize = 4;
const MAX_TOTAL: usize = 8;

struct SubagentEntry {
    pub cancel: CancellationToken,
    pub parent_stream_cancel: CancellationToken,
    pub parent_session_id: String,
    pub run_id: String,
    pub execution_id: String,
    pub initial_prompt_hash: Option<[u8; 32]>,
    pub delivered_prompt_hashes: Vec<[u8; 32]>,
}

pub struct RegisteredSubagent {
    pub run_id: String,
    pub execution_id: String,
}

struct RegistryState {
    entries: HashMap<String, SubagentEntry>,
    run_ids: HashMap<String, String>,
    run_claims: HashMap<String, usize>,
    terminal_signals: HashMap<String, SubagentTerminalNotifier>,
}

static REGISTRY: LazyLock<Mutex<RegistryState>> = LazyLock::new(|| {
    Mutex::new(RegistryState {
        entries: HashMap::new(),
        run_ids: HashMap::new(),
        run_claims: HashMap::new(),
        terminal_signals: HashMap::new(),
    })
});

pub async fn get_or_create_run_id(parent_id: &str) -> String {
    let mut state = REGISTRY.lock().await;
    let run_id = state
        .run_ids
        .entry(parent_id.to_string())
        .or_insert_with(|| uuid::Uuid::new_v4().to_string())
        .clone();
    *state.run_claims.entry(parent_id.to_string()).or_insert(0) += 1;
    run_id
}

#[cfg(test)]
pub async fn register(
    parent_id: &str,
    child_id: &str,
    cancel: CancellationToken,
) -> Result<String, String> {
    register_execution(parent_id, child_id, cancel)
        .await
        .map(|registered| registered.run_id)
}

#[cfg(test)]
pub async fn register_execution(
    parent_id: &str,
    child_id: &str,
    cancel: CancellationToken,
) -> Result<RegisteredSubagent, String> {
    register_execution_with_initial_prompt(parent_id, child_id, cancel, None).await
}

#[cfg(test)]
pub async fn register_execution_with_initial_prompt(
    parent_id: &str,
    child_id: &str,
    cancel: CancellationToken,
    initial_prompt: Option<&str>,
) -> Result<RegisteredSubagent, String> {
    register_execution_for_parent_stream(
        parent_id,
        child_id,
        cancel,
        initial_prompt,
        &CancellationToken::new(),
    )
    .await
}

pub async fn register_execution_for_parent_stream(
    parent_id: &str,
    child_id: &str,
    cancel: CancellationToken,
    initial_prompt: Option<&str>,
    parent_cancel: &CancellationToken,
) -> Result<RegisteredSubagent, String> {
    let mut state = REGISTRY.lock().await;
    if parent_cancel.is_cancelled() {
        return Err("Délégation annulée.".to_string());
    }
    let parent_count = state
        .entries
        .values()
        .filter(|e| e.parent_session_id == parent_id)
        .count();
    if let Some(error) = registration_capacity_error(state.entries.len(), parent_count) {
        return Err(error);
    }
    if state.entries.contains_key(child_id) {
        return Err("Ce sous-agent est déjà en cours.".to_string());
    }
    ensure_parent_signal_locked(&mut state, parent_id)?;
    let run_id = state
        .run_ids
        .entry(parent_id.to_string())
        .or_insert_with(|| uuid::Uuid::new_v4().to_string())
        .clone();
    release_claim_locked(&mut state, parent_id);
    let execution_id = uuid::Uuid::new_v4().to_string();
    state.entries.insert(
        child_id.to_string(),
        SubagentEntry {
            cancel,
            parent_stream_cancel: parent_cancel.clone(),
            parent_session_id: parent_id.to_string(),
            run_id: run_id.clone(),
            execution_id: execution_id.clone(),
            initial_prompt_hash: initial_prompt.map(prompt_hash),
            delivered_prompt_hashes: Vec::new(),
        },
    );
    Ok(RegisteredSubagent {
        run_id,
        execution_id,
    })
}

pub(super) fn capacity_error(total: usize, parent_count: usize) -> Option<String> {
    if total >= MAX_TOTAL {
        return Some(format!("Limite de {MAX_TOTAL} sous-agents actifs atteinte"));
    }
    if parent_count >= MAX_PER_PARENT {
        return Some(format!(
            "Limite de {MAX_PER_PARENT} sous-agents par session atteinte"
        ));
    }
    None
}

fn registration_capacity_error(total: usize, parent_count: usize) -> Option<String> {
    #[cfg(not(test))]
    let effective_total = total;
    #[cfg(test)]
    let effective_total = {
        let _ = total;
        parent_count
    };
    capacity_error(effective_total, parent_count)
}

pub async fn unregister(child_id: &str) {
    let mut state = REGISTRY.lock().await;
    if let Some(entry) = state.entries.remove(child_id) {
        cleanup_parent_locked(&mut state, &entry.parent_session_id, true);
    }
}

pub async fn release_run_claim(parent_id: &str, run_id: &str) {
    let mut state = REGISTRY.lock().await;
    if state.run_ids.get(parent_id).map(String::as_str) != Some(run_id) {
        return;
    }
    release_claim_locked(&mut state, parent_id);
    let has_entries = state
        .entries
        .values()
        .any(|entry| entry.parent_session_id == parent_id);
    let has_claims = state.run_claims.get(parent_id).copied().unwrap_or(0) > 0;
    if !has_entries && !has_claims {
        state.run_ids.remove(parent_id);
    }
}

fn release_claim_locked(state: &mut RegistryState, parent_id: &str) {
    let Some(count) = state.run_claims.get_mut(parent_id) else {
        return;
    };
    if *count > 1 {
        *count -= 1;
    } else {
        state.run_claims.remove(parent_id);
    }
}

include!("subagent_registry_execution.rs");

pub async fn active_children_for_parent(parent_id: &str) -> Vec<String> {
    REGISTRY
        .lock()
        .await
        .entries
        .iter()
        .filter(|(_, entry)| entry.parent_session_id == parent_id)
        .map(|(child_id, _)| child_id.clone())
        .collect()
}

include!("subagent_registry_terminal.rs");
