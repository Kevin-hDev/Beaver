use super::{
    session_types::{BrowserTabState, MAX_BROWSER_TABS},
    tab_id::validate_tab_id,
};

pub(super) fn ordered_tabs(
    tabs: &[BrowserTabState],
    ordered_ids: &[String],
) -> Result<Vec<BrowserTabState>, ()> {
    if !(1..=MAX_BROWSER_TABS).contains(&tabs.len()) || ordered_ids.len() != tabs.len() {
        return Err(());
    }

    let mut ordered = Vec::with_capacity(ordered_ids.len());
    let mut seen = Vec::with_capacity(ordered_ids.len());
    for id in ordered_ids {
        validate_tab_id(id)?;
        if seen.contains(&id.as_str()) {
            return Err(());
        }
        let tab = tabs
            .iter()
            .find(|tab| tab.id.as_str() == id.as_str())
            .ok_or(())?;
        seen.push(id.as_str());
        ordered.push(tab.clone());
    }
    Ok(ordered)
}
