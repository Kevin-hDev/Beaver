use std::collections::VecDeque;
use std::sync::{Arc, LazyLock, Mutex};

use super::tool_bash_session::ShellSession;

const MAX_SESSIONS: usize = 64;

static SESSIONS: LazyLock<Mutex<VecDeque<Arc<ShellSession>>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub fn insert(session: Arc<ShellSession>) -> Result<(), String> {
    let mut sessions = lock_sessions();
    while sessions.len() >= MAX_SESSIONS {
        let Some(position) = sessions.iter().position(|candidate| candidate.is_done()) else {
            return Err("Trop de processus shell actifs.".to_string());
        };
        sessions.remove(position);
    }
    sessions.push_back(session);
    Ok(())
}

pub fn get(process_id: &str, owner_session_id: &str) -> Result<Arc<ShellSession>, String> {
    let parsed = uuid::Uuid::parse_str(process_id)
        .map_err(|_| "Session shell introuvable.".to_string())?;
    let process_id = parsed.to_string();
    let mut sessions = lock_sessions();
    let Some(position) = sessions.iter().position(|session| {
        session.id() == process_id && session.owner_session_id() == owner_session_id
    }) else {
        return Err("Session shell introuvable.".to_string());
    };
    let session = sessions
        .remove(position)
        .ok_or_else(|| "Session shell introuvable.".to_string())?;
    sessions.push_back(Arc::clone(&session));
    Ok(session)
}

pub fn remove(process_id: &str) {
    let mut sessions = lock_sessions();
    sessions.retain(|session| session.id() != process_id);
}

pub async fn stop_all() {
    let sessions = {
        let mut sessions = lock_sessions();
        sessions.drain(..).collect::<Vec<_>>()
    };
    for session in &sessions {
        session.cancel();
    }
    let finished = async {
        while sessions.iter().any(|session| !session.is_done()) {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    };
    if tokio::time::timeout(std::time::Duration::from_secs(3), finished)
        .await
        .is_err()
    {
        for session in sessions.iter().filter(|session| !session.is_done()) {
            super::tool_bash_platform::terminate_process_tree(session.pid()).await;
        }
    }
}

fn lock_sessions() -> std::sync::MutexGuard<'static, VecDeque<Arc<ShellSession>>> {
    SESSIONS.lock().unwrap_or_else(|error| error.into_inner())
}
