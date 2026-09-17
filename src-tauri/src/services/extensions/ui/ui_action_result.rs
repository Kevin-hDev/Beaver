use serde_json::Value;
use std::collections::HashSet;

pub(super) struct ValidatedUiActionResult {
    pub value: Value,
    pub action_ids: Vec<String>,
}

pub(super) fn validate(owner: &str, value: Value) -> Result<ValidatedUiActionResult, String> {
    if serde_json::to_vec(&value).map_err(|_| invalid())?.len()
        > super::ui_contract::MAX_ACTION_RESULT_BYTES
    {
        return Err(limit());
    }
    let object = value.as_object().ok_or_else(invalid)?;
    let mut actions = HashSet::new();
    match super::ui_view_validation::string(object, "type")? {
        "notification" => {
            super::ui_view_validation::exact(object, &["type", "level", "message"])?;
            let level = super::ui_view_validation::string(object, "level")?;
            if !["info", "success", "warning", "error"].contains(&level) {
                return Err(invalid());
            }
            super::ui_view_validation::localized(object.get("message").ok_or_else(invalid)?)?;
        }
        "view" => {
            super::ui_view_validation::exact(object, &["type", "view"])?;
            super::ui_view_validation::validate_view(
                owner,
                object.get("view").ok_or_else(invalid)?,
                &mut actions,
            )?;
        }
        _ => return Err(invalid()),
    }
    let mut action_ids = actions.into_iter().collect::<Vec<_>>();
    action_ids.sort_unstable();
    Ok(ValidatedUiActionResult { value, action_ids })
}

fn invalid() -> String {
    "ui_action_result_invalid".to_string()
}

fn limit() -> String {
    "ui_limit_exceeded".to_string()
}
