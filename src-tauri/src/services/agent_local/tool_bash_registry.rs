use std::collections::VecDeque;
use std::sync::{Arc, LazyLock, Mutex};

use super::tool_bash_session::ShellSession;

const MAX_SESSIONS: usize = 64;

struct RegisteredSession {
    session: Arc<ShellSession>,
    command: Arc<str>,
}

static SESSIONS: LazyLock<Mutex<VecDeque<RegisteredSession>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub fn insert(session: Arc<ShellSession>, command: &str) -> Result<(), String> {
    let mut sessions = lock_sessions();
    while sessions.len() >= MAX_SESSIONS {
        let Some(position) = sessions
            .iter()
            .position(|candidate| candidate.session.is_done())
        else {
            return Err("Trop de processus shell actifs.".to_string());
        };
        sessions.remove(position);
    }
    sessions.push_back(RegisteredSession {
        session,
        command: Arc::from(command),
    });
    Ok(())
}

pub fn get(process_id: &str, owner_session_id: &str) -> Result<Arc<ShellSession>, String> {
    let parsed = uuid::Uuid::parse_str(process_id)
        .map_err(|_| "Session shell introuvable.".to_string())?;
    let process_id = parsed.to_string();
    let mut sessions = lock_sessions();
    let Some(position) = sessions.iter().position(|entry| {
        entry.session.id() == process_id && entry.session.owner_session_id() == owner_session_id
    }) else {
        return Err("Session shell introuvable.".to_string());
    };
    let entry = sessions
        .remove(position)
        .ok_or_else(|| "Session shell introuvable.".to_string())?;
    let session = Arc::clone(&entry.session);
    sessions.push_back(entry);
    Ok(session)
}

pub fn command(process_id: &str, owner_session_id: &str) -> Result<Arc<str>, String> {
    let parsed = uuid::Uuid::parse_str(process_id)
        .map_err(|_| "Session shell introuvable.".to_string())?;
    let process_id = parsed.to_string();
    let sessions = lock_sessions();
    sessions
        .iter()
        .find(|entry| {
            entry.session.id() == process_id
                && entry.session.owner_session_id() == owner_session_id
        })
        .map(|entry| Arc::clone(&entry.command))
        .ok_or_else(|| "Session shell introuvable.".to_string())
}

pub fn remove(process_id: &str) {
    let mut sessions = lock_sessions();
    sessions.retain(|entry| entry.session.id() != process_id);
}

pub async fn stop_all() {
    let sessions = {
        let mut sessions = lock_sessions();
        sessions
            .drain(..)
            .map(|entry| entry.session)
            .collect::<Vec<_>>()
    };
    for session in &sessions {
        session.stop();
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

fn lock_sessions() -> std::sync::MutexGuard<'static, VecDeque<RegisteredSession>> {
    SESSIONS.lock().unwrap_or_else(|error| error.into_inner())
}
