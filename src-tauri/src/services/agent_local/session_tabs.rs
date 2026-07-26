use super::session_store::{get, validate_session_id};
use super::session_tabs_file::{read_file, write_file, TABS_LOCK};
use super::session_tabs_state::{
    clean_label, first_free_branch_number, normalize_tabs, validate_tabs, MAIN_TAB_ID,
    MAX_TABS_PER_SESSION,
};
use super::types_session::CloneMode;

pub use super::session_tabs_git::{
    clear_git_branch_for_sessions, get_main_checkpoint_branch, get_tab,
    replace_main_checkpoint_branch, set_clone_git_branch,
};
pub use super::session_tabs_state::{SessionTab, SessionTabs};

pub async fn list(root_session_id: &str) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let original = file.sessions.get(root_session_id).cloned();
    let mut tabs = normalize_tabs(root_session_id, original.clone());
    super::session_tabs_git::sync_git_branches_from_sessions(root_session_id, &mut tabs).await?;
    if original.is_some() && original.as_ref() != Some(&tabs) {
        file.sessions
            .insert(root_session_id.to_string(), tabs.clone());
        write_file(&file).await?;
    }
    Ok(tabs)
}

pub async fn save_tabs(root_session_id: &str, tabs: SessionTabs) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut normalized = normalize_tabs(root_session_id, Some(tabs));
    super::session_tabs_git::sync_git_branches_from_sessions(root_session_id, &mut normalized)
        .await?;
    validate_tabs(root_session_id, &normalized)?;
    file.sessions
        .insert(root_session_id.to_string(), normalized.clone());
    write_file(&file).await?;
    Ok(normalized)
}

pub async fn add_clone_tab(
    root_session_id: &str,
    clone_session_id: &str,
    parent_message_id: &str,
    mode: CloneMode,
) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    validate_session_id(clone_session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let clone = get(clone_session_id).await?;
    let actual_root = super::clone_roots::resolve_source_root_id(&clone).await?;
    if actual_root != root_session_id {
        return Err("Action impossible".into());
    }
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    if let Some(existing) = tabs
        .tabs
        .iter()
        .find(|tab| tab.session_id == clone_session_id)
    {
        tabs.active_tab_id = existing.tab_id.clone();
    } else {
        push_clone_tab(
            root_session_id,
            clone_session_id,
            parent_message_id,
            mode,
            clone.git_branch,
            &mut tabs,
        )?;
    }
    file.sessions
        .insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    Ok(tabs)
}

pub async fn close_tab(root_session_id: &str, tab_id: &str) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    if tab_id == MAIN_TAB_ID {
        return Err("Action impossible".into());
    }
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    let removed = remove_tab(&mut tabs, tab_id)?;
    file.sessions
        .insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    super::session_archive::archive(&removed.session_id).await?;
    Ok(tabs)
}

pub async fn rename_tab(
    root_session_id: &str,
    tab_id: &str,
    label: &str,
) -> Result<SessionTabs, String> {
    validate_session_id(root_session_id)?;
    let label = clean_label(label)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut tabs = normalize_tabs(root_session_id, file.sessions.get(root_session_id).cloned());
    let Some(tab) = tabs.tabs.iter_mut().find(|tab| tab.tab_id == tab_id) else {
        return Err("Action impossible".into());
    };
    tab.label = label;
    file.sessions
        .insert(root_session_id.to_string(), tabs.clone());
    write_file(&file).await?;
    Ok(tabs)
}

pub async fn remove_session_from_tabs(session_id: &str) -> Result<(), String> {
    validate_session_id(session_id)?;
    let _guard = TABS_LOCK.lock().await;
    let mut file = read_file().await?;
    let mut empty_roots = Vec::new();
    for (root_id, tabs) in &mut file.sessions {
        tabs.tabs.retain(|tab| tab.session_id != session_id);
        if tabs.tabs.is_empty() || root_id == session_id {
            empty_roots.push(root_id.clone());
        } else if !tabs.tabs.iter().any(|tab| tab.tab_id == tabs.active_tab_id) {
            tabs.active_tab_id = MAIN_TAB_ID.to_string();
        }
    }
    for root_id in empty_roots {
        file.sessions.remove(&root_id);
    }
    write_file(&file).await
}

pub async fn ensure_clone_tab(root_session_id: &str, clone_session_id: &str) -> Result<(), String> {
    let clone = get(clone_session_id).await?;
    let parent_message_id = clone
        .clone_parent_message_id
        .as_deref()
        .ok_or_else(|| "Action impossible".to_string())?;
    let mode = clone.clone_mode.clone().unwrap_or(CloneMode::Cut);
    add_clone_tab(root_session_id, clone_session_id, parent_message_id, mode).await?;
    Ok(())
}

fn push_clone_tab(
    root_session_id: &str,
    clone_session_id: &str,
    parent_message_id: &str,
    mode: CloneMode,
    git_branch: Option<String>,
    tabs: &mut SessionTabs,
) -> Result<(), String> {
    if tabs.tabs.len() >= MAX_TABS_PER_SESSION {
        return Err("Nombre maximum d'onglets atteint".into());
    }
    let branch_number = first_free_branch_number(tabs)?;
    let tab_id = format!("branch-{branch_number}");
    tabs.tabs.push(SessionTab {
        tab_id: tab_id.clone(),
        session_id: clone_session_id.to_string(),
        label: format!("Branche {branch_number}"),
        is_main: false,
        clone_parent_session_id: Some(root_session_id.to_string()),
        clone_parent_message_id: Some(parent_message_id.to_string()),
        clone_mode: Some(mode),
        git_branch,
    });
    tabs.active_tab_id = tab_id;
    Ok(())
}

fn remove_tab(tabs: &mut SessionTabs, tab_id: &str) -> Result<SessionTab, String> {
    let Some(pos) = tabs.tabs.iter().position(|tab| tab.tab_id == tab_id) else {
        return Err("Action impossible".into());
    };
    let removed = tabs.tabs.remove(pos);
    tabs.active_tab_id = MAIN_TAB_ID.to_string();
    Ok(removed)
}
