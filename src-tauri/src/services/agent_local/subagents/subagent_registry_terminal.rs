pub(super) const PRODUCTION_MAX_TERMINAL_PARENTS: usize = 16;
#[cfg(not(test))]
const MAX_TERMINAL_PARENTS: usize = PRODUCTION_MAX_TERMINAL_PARENTS;
// Unit tests share this global registry and run concurrently in the Rust harness.
#[cfg(test)]
const MAX_TERMINAL_PARENTS: usize = 64;
const TERMINAL_STATE_ERROR: &str = "État terminal de sous-agent indisponible";

pub struct ParentRegistrySnapshot {
    pub active_child_ids: Vec<String>,
    pub terminal_state: Option<SubagentTerminalState>,
}

fn parent_has_entries(state: &RegistryState, parent_id: &str) -> bool {
    state
        .entries
        .values()
        .any(|entry| entry.parent_session_id == parent_id)
}

fn ensure_parent_signal_locked(
    state: &mut RegistryState,
    parent_id: &str,
) -> Result<(), String> {
    if let Some(signal) = state.terminal_signals.get(parent_id) {
        if !parent_has_entries(state, parent_id)
            && signal.state().sequence > 0
            && signal.state().report_persistence_failed
        {
            return Err("Un résultat de sous-agent reste à traiter".to_string());
        }
        return Ok(());
    }
    if state.terminal_signals.len() >= MAX_TERMINAL_PARENTS {
        return Err(TERMINAL_STATE_ERROR.to_string());
    }
    state.terminal_signals.insert(
        parent_id.to_string(),
        super::subagent_terminal_signal::notifier(),
    );
    Ok(())
}

fn cleanup_parent_locked(state: &mut RegistryState, parent_id: &str, drop_idle_signal: bool) {
    let has_entries = parent_has_entries(state, parent_id);
    let has_claims = state.run_claims.get(parent_id).copied().unwrap_or(0) > 0;
    if !has_entries && !has_claims {
        state.run_ids.remove(parent_id);
    }
    if drop_idle_signal && !has_entries {
        let idle = state
            .terminal_signals
            .get(parent_id)
            .is_some_and(|signal| signal.state().sequence == 0);
        if idle {
            state.terminal_signals.remove(parent_id);
        }
    }
}

#[cfg(test)]
pub async fn complete_child(
    child_id: &str,
    kind: SubagentTerminalKind,
) -> Result<(), String> {
    let mut state = REGISTRY.lock().await;
    let parent_id = state
        .entries
        .get(child_id)
        .map(|entry| entry.parent_session_id.clone())
        .ok_or_else(|| TERMINAL_STATE_ERROR.to_string())?;
    let signal = state
        .terminal_signals
        .get(&parent_id)
        .cloned()
        .ok_or_else(|| TERMINAL_STATE_ERROR.to_string())?;
    state.entries.remove(child_id);
    signal.notify(kind);
    cleanup_parent_locked(&mut state, &parent_id, false);
    Ok(())
}

pub async fn complete_child_or_parent(
    parent_id: &str,
    child_id: &str,
    kind: SubagentTerminalKind,
) -> Result<(), String> {
    let mut state = REGISTRY.lock().await;
    if let Some(entry) = state.entries.get(child_id) {
        if entry.parent_session_id != parent_id {
            return Err(TERMINAL_STATE_ERROR.to_string());
        }
        state.entries.remove(child_id);
    }
    ensure_parent_signal_locked(&mut state, parent_id)?;
    let signal = state
        .terminal_signals
        .get(parent_id)
        .cloned()
        .ok_or_else(|| TERMINAL_STATE_ERROR.to_string())?;
    signal.notify(kind);
    cleanup_parent_locked(&mut state, parent_id, false);
    Ok(())
}

pub async fn subscribe_for_parent(
    parent_id: &str,
) -> Option<tokio::sync::watch::Receiver<SubagentTerminalState>> {
    REGISTRY
        .lock()
        .await
        .terminal_signals
        .get(parent_id)
        .map(SubagentTerminalNotifier::subscribe)
}

pub async fn terminal_state_for_parent(parent_id: &str) -> Option<SubagentTerminalState> {
    REGISTRY
        .lock()
        .await
        .terminal_signals
        .get(parent_id)
        .map(SubagentTerminalNotifier::state)
}

pub async fn parent_snapshot(parent_id: &str) -> ParentRegistrySnapshot {
    let state = REGISTRY.lock().await;
    ParentRegistrySnapshot {
        active_child_ids: state
            .entries
            .iter()
            .filter(|(_, entry)| entry.parent_session_id == parent_id)
            .map(|(child_id, _)| child_id.clone())
            .collect(),
        terminal_state: state
            .terminal_signals
            .get(parent_id)
            .map(SubagentTerminalNotifier::state),
    }
}

pub async fn consume_terminal(parent_id: &str, generation: u64, sequence: u64) -> bool {
    let mut state = REGISTRY.lock().await;
    let Some(signal) = state.terminal_signals.get(parent_id).cloned() else {
        return false;
    };
    let current = signal.state();
    if current.generation != generation
        || current.sequence != sequence
        || current.sequence == 0
        || current.report_persistence_failed
    {
        return false;
    }
    if parent_has_entries(&state, parent_id) {
        signal.reset();
    } else {
        state.terminal_signals.remove(parent_id);
    }
    true
}
