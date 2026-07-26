use super::{
    session_model::SessionModel,
    session_types::{BrowserRuntimeTabUpdate, MAX_TITLE_CHARS},
    url_policy::validate_browser_url,
};

impl SessionModel {
    pub(super) fn update_runtime(
        &mut self,
        id: &str,
        update: &BrowserRuntimeTabUpdate,
    ) -> Result<bool, ()> {
        let url = update
            .url
            .as_deref()
            .map(validate_browser_url)
            .transpose()?
            .map(|value| value.as_str().to_owned());
        let tab = self.tab_mut(id)?;
        let title = update.title.as_deref().map(clean_title);
        let changed = assign_if_some(&mut tab.title, title)
            | assign_if_some(&mut tab.url, url.map(Some))
            | assign_if_some(&mut tab.loading, update.loading)
            | assign_if_some(&mut tab.can_go_back, update.can_go_back)
            | assign_if_some(&mut tab.can_go_forward, update.can_go_forward)
            | std::mem::replace(&mut tab.released, false);
        if changed {
            self.bump()?;
        }
        Ok(changed)
    }

    pub(super) fn mark_released(&mut self, id: &str) -> Result<bool, ()> {
        let tab = self.tab_mut(id)?;
        let changed = !tab.released || tab.loading || tab.can_go_back || tab.can_go_forward;
        tab.released = true;
        tab.loading = false;
        tab.can_go_back = false;
        tab.can_go_forward = false;
        if changed {
            self.bump()?;
        }
        Ok(changed)
    }
}

fn assign_if_some<T: PartialEq>(target: &mut T, value: Option<T>) -> bool {
    let Some(value) = value else {
        return false;
    };
    if *target == value {
        return false;
    }
    *target = value;
    true
}

fn clean_title(raw: &str) -> String {
    let mut title = String::new();
    let mut length = 0;
    let mut pending_space = false;
    for character in raw.chars() {
        if length == MAX_TITLE_CHARS {
            break;
        }
        if character.is_whitespace() || character.is_control() {
            pending_space = !title.is_empty();
            continue;
        }
        if pending_space && length < MAX_TITLE_CHARS {
            title.push(' ');
            length += 1;
            pending_space = false;
        }
        if length < MAX_TITLE_CHARS {
            title.push(character);
            length += 1;
        }
    }
    title
}
