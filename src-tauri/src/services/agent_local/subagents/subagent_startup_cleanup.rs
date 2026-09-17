//! Cleanup au démarrage des sous-agents orphelins.
//!
//! Au démarrage, le registry des sous-agents actifs est vide (LazyLock).
//! Toute session `running` antérieure au démarrage est reclassée `interrupted`.
//! Les captures de worktrees ayant échoué sont retentées pour les statuts
//! `running`, `interrupted` et `failed` sans effacer le travail récupérable.

use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};

use super::project_store;
use super::session_index;
use super::session_store;
use super::subagent_status;
use super::types_session::AgentSessionMeta;

const PRUNE_TIMEOUT_SECS: u64 = 3;

/// Nettoie les sous-agents orphelins détectés au démarrage.
///
/// Sans danger : le registry en mémoire étant vide au boot, toute session
/// "running" antérieure au démarrage est orpheline. Les erreurs individuelles
/// sont loggées et n'interrompent pas le cleanup global.
pub async fn cleanup_orphans(startup_cutoff: DateTime<Utc>) {
    let sessions_dir = crate::services::paths::data_dir().join("agent-sessions");
    let cleaned = match cleanup_orphans_in_dir(&sessions_dir, startup_cutoff, true).await {
        Ok(count) => count,
        Err(_) => {
            ::log::error!("[startup-cleanup] cleanup sessions impossible");
            return;
        }
    };

    let pruned = prune_project_worktrees().await;

    ::log::info!(
        "[startup-cleanup] {cleaned} sous-agent(s) orphelin(s) nettoyé(s), {pruned} projet(s) pruné(s)"
    );
}

pub(crate) async fn cleanup_orphans_in_dir(
    sessions_dir: &Path,
    startup_cutoff: DateTime<Utc>,
    remove_worktrees: bool,
) -> Result<usize, String> {
    let global = sessions_dir == global_sessions_dir();
    let metas = if global {
        session_index::rebuild_index().await?
    } else {
        session_index::rebuild_index_from(sessions_dir).await?
    };
    let mut cleaned = 0usize;

    for meta in orphan_candidates(&metas, startup_cutoff) {
        let mut session = match session_store::read_from_dir(sessions_dir, &meta.id).await {
            Ok(session) => session,
            Err(_) => {
                ::log::warn!("[startup-cleanup] lecture session impossible");
                continue;
            }
        };
        let mut changed = false;
        if session.subagent_status.as_deref() == Some(subagent_status::RUNNING) {
            session.subagent_status = Some(subagent_status::INTERRUPTED.to_string());
            if write_session(sessions_dir, &session, global).await.is_err() {
                ::log::warn!("[startup-cleanup] mise à jour de session impossible");
                continue;
            }
            changed = true;
        }

        if remove_worktrees && session.subagent_worktree.is_some() {
            match super::subagent_task_change::recover_and_remove_orphan(&session).await {
                Ok(()) => {
                    session.subagent_worktree = None;
                    if write_session(sessions_dir, &session, global).await.is_err() {
                        ::log::warn!("[startup-cleanup] finalisation worktree impossible");
                    } else {
                        changed = true;
                    }
                }
                Err(_) => ::log::warn!("[startup-cleanup] récupération worktree impossible"),
            }
        }
        cleaned += usize::from(changed);
    }

    if cleaned > 0 && !global {
        session_index::rebuild_index_from(sessions_dir).await?;
    }

    Ok(cleaned)
}

fn global_sessions_dir() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("agent-sessions")
}

async fn write_session(
    sessions_dir: &Path,
    session: &super::types_session::AgentSession,
    global: bool,
) -> Result<(), String> {
    if global {
        session_store::save(session).await
    } else {
        session_store::write_to_dir(sessions_dir, session).await
    }
}

fn orphan_candidates(
    metas: &[AgentSessionMeta],
    startup_cutoff: DateTime<Utc>,
) -> impl Iterator<Item = &AgentSessionMeta> {
    metas
        .iter()
        .filter(move |meta| is_orphan_candidate(meta, startup_cutoff))
        .take(super::session_limits::MAX_SESSION_FILES)
}

fn is_orphan_candidate(meta: &AgentSessionMeta, startup_cutoff: DateTime<Utc>) -> bool {
    if meta.parent_session_id.is_none() {
        return false;
    }
    match meta.subagent_status.as_deref() {
        Some(subagent_status::RUNNING) => {
            meta.updated_at.unwrap_or(meta.created_at) <= startup_cutoff
        }
        Some(subagent_status::INTERRUPTED | subagent_status::FAILED) => true,
        _ => false,
    }
}

/// Lance `git worktree prune` sur chaque projet connu, en parallèle et avec timeout.
/// Les projets inaccessibles sont ignorés silencieusement.
async fn prune_project_worktrees() -> usize {
    let projects = match project_store::list().await {
        Ok(projects) => projects,
        Err(_) => {
            ::log::warn!("[startup-cleanup] lecture des projets impossible");
            return 0;
        }
    };
    let mut pruned = 0usize;

    for project in projects {
        let path = std::path::PathBuf::from(&project.path);
        if !path.is_dir() {
            continue;
        }
        if prune_one_project(&path).await {
            pruned += 1;
        }
    }
    pruned
}

async fn prune_one_project(path: &std::path::Path) -> bool {
    let mut command = crate::services::background_command::new_tokio("git");
    command
        .args(["-C"])
        .arg(path)
        .args(["worktree", "prune"])
        .kill_on_drop(true);
    let fut = command.output();

    match tokio::time::timeout(Duration::from_secs(PRUNE_TIMEOUT_SECS), fut).await {
        Ok(Ok(output)) if output.status.success() => true,
        Ok(Ok(_)) => {
            ::log::warn!("[startup-cleanup] git worktree prune échoué");
            false
        }
        Ok(Err(_)) => {
            ::log::warn!("[startup-cleanup] git worktree prune indisponible");
            false
        }
        Err(_) => {
            ::log::warn!("[startup-cleanup] git worktree prune timeout");
            false
        }
    }
}

#[cfg(test)]
#[path = "subagent_startup_cleanup_tests.rs"]
mod tests;
