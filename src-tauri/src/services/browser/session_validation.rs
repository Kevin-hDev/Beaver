use super::{
    session_types::{PersistedBrowserSession, MAX_BROWSER_TABS, MAX_TITLE_CHARS, SESSION_VERSION},
    tab_id::validate_tab_id,
    url_policy::validate_browser_url,
};
use std::collections::HashSet;

pub(super) fn validate_persisted(value: &PersistedBrowserSession) -> Result<(), ()> {
    if value.version != SESSION_VERSION
        || value.state.generation == 0
        || value.state.tabs.is_empty()
        || value.state.tabs.len() > MAX_BROWSER_TABS
        || value.recency.len() != value.state.tabs.len()
    {
        return Err(());
    }
    let ids: HashSet<&str> = value.state.tabs.iter().map(|tab| tab.id.as_str()).collect();
    if ids.len() != value.state.tabs.len()
        || !ids.contains(value.state.active_tab_id.as_str())
        || value.recency.iter().any(|id| !ids.contains(id.as_str()))
        || value.recency.iter().collect::<HashSet<_>>().len() != value.recency.len()
    {
        return Err(());
    }
    for tab in &value.state.tabs {
        validate_tab_id(&tab.id)?;
        if tab.title.chars().count() > MAX_TITLE_CHARS {
            return Err(());
        }
        if let Some(url) = tab.url.as_deref() {
            validate_browser_url(url)?;
        }
    }
    Ok(())
}
