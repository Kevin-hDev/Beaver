use super::session_store::validate_session_id;
use super::session_tabs_state::{normalize_tabs, MAIN_TAB_ID};
use super::{
    session_store,
    session_tabs::{SessionTab, SessionTabs},
    session_tabs_file::{read_file, write_file, TABS_LOCK},
};
use crate::services::git::branch;
use std::collections::HashSet;

pub async fn get_tab(root_session_id: &str, tab_id: &str) -> Result<SessionTab, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let file = read_file().await?;
    let tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    tabs.tabs
        .into_iter()
        .find(|tab| tab.tab_id == tab_id)
        .ok_or_else(|| "Action impossible".to_string())
}

/// Lit le `main_checkpoint_branch` persisté dans `session-tabs.json` pour la session
/// principale donnée. Source de vérité pour le fallback auto lors du nettoyage de
/// branche (sans dépendre d'une valeur envoyée par le frontend).
pub async fn get_main_checkpoint_branch(root_session_id: &str) -> Result<Option<String>, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let file = read_file().await?;
    let tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    Ok(tabs.main_checkpoint_branch)
}

pub async fn set_clone_git_branch(
    root_session_id: &str,
    clone_session_id: &str,
    git_branch: Option<String>,
) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    validate_session_id(clone_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    let Some(tab) = tabs
        .tabs
        .iter_mut()
        .find(|tab| !tab.is_main && tab.session_id == clone_session_id)
    else {
        return Err("Action impossible".into());
    };
    tab.git_branch = git_branch;
    file.sessions
        .insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    Ok(tabs)
}

pub async fn clear_git_branch_for_sessions(session_ids: &[String]) -> Result<(), String> {
    let ids: HashSet<&str> = session_ids.iter().map(String::as_str).collect();
    for session_id in &ids {
        validate_session_id(session_id)?;
    }
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    for tabs in file.sessions.values_mut() {
        for tab in &mut tabs.tabs {
            if ids.contains(tab.session_id.as_str()) {
                tab.git_branch = None;
            }
        }
    }
    write_file(&file).await
}

pub async fn replace_main_checkpoint_branch(
    deleted_branch: &str,
    replacement_branch: &str,
) -> Result<(), String> {
    branch::validate_branch_name(deleted_branch).map_err(|_| "Action impossible".to_string())?;
    branch::validate_branch_name(replacement_branch)
        .map_err(|_| "Action impossible".to_string())?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    for tabs in file.sessions.values_mut() {
        if tabs.main_checkpoint_branch.as_deref() == Some(deleted_branch) {
            tabs.main_checkpoint_branch = Some(replacement_branch.to_string());
        }
    }
    write_file(&file).await
}

pub async fn sync_git_branches_from_sessions(
    root_session_id: &str,
    tabs: &mut SessionTabs,
) -> Result<(), String> {
    let metas = super::session_index::read_index().await?;
    let mut synced_tabs = Vec::with_capacity(tabs.tabs.len());
    for mut tab in tabs.tabs.drain(..) {
        if tab.is_main {
            tab.git_branch = None;
            synced_tabs.push(tab);
            continue;
        }
        let Ok(mut session) = session_store::get(&tab.session_id).await else {
            continue;
        };
        let Ok(root_id) = super::clone_roots::root_id_from_metas(
            &metas,
            &session.id,
            session.clone_parent_session_id.as_deref(),
        ) else {
            continue;
        };
        if root_id != root_session_id {
            continue;
        }
        if let Some(branch) = session.git_branch.clone() {
            if super::clone_roots::git_branch_shared_with_ancestor(
                &metas,
                &session.id,
                session.clone_parent_session_id.as_deref(),
                &branch,
            ) {
                session.git_branch = None;
                let _ = session_store::save(&session).await;
            }
        }
        tab.clone_parent_session_id = Some(root_session_id.to_string());
        tab.git_branch = session.git_branch;
        synced_tabs.push(tab);
    }
    tabs.tabs = synced_tabs;
    if !tabs.tabs.iter().any(|tab| tab.tab_id == tabs.active_tab_id) {
        tabs.active_tab_id = MAIN_TAB_ID.to_string();
    }
    Ok(())
}
