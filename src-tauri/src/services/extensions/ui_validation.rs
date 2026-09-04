use super::host_identity::HostIdentity;
use super::types::{ExtensionApiLevel, ExtensionUiManifest, ExtensionUiMode};
use super::ui_types::UiCatalogEntry;
use serde_json::{Map, Value};
use std::collections::HashSet;

pub(super) fn catalog(
    identity: &HostIdentity,
    extension_id: &str,
    _api_level: &ExtensionApiLevel,
    manifest: Option<&ExtensionUiManifest>,
    contributions: Vec<Value>,
) -> Result<Vec<UiCatalogEntry>, String> {
    authorize(identity, extension_id)?;
    let Some(manifest) = manifest else {
        return contributions.is_empty().then(Vec::new).ok_or_else(invalid);
    };
    if manifest.api_version != super::ui_contract::EXTENSION_UI_API_VERSION {
        return Err(invalid());
    }
    if manifest.mode == ExtensionUiMode::Advanced {
        // UI-P5 prendra en charge l'artefact avancé. Jusque-là sa déclaration
        // valide reste inerte et ne peut pas emprunter le canal standard.
        return contributions.is_empty().then(Vec::new).ok_or_else(invalid);
    }
    if manifest.entry.is_some()
        || contributions.len() > super::ui_contract::MAX_CONTRIBUTIONS_PER_EXTENSION
    {
        return Err(invalid());
    }
    let mut contributions = contributions;
    for contribution in &mut contributions {
        super::ui_normalization::contribution(extension_id, contribution)?;
    }
    let bytes = serde_json::to_vec(&contributions)
        .map_err(|_| invalid())?
        .len();
    if bytes > super::ui_contract::MAX_UI_BYTES_PER_EXTENSION {
        return Err(limit());
    }
    let mut ids = HashSet::new();
    let mut action_ids = HashSet::new();
    let mut themes = 0usize;
    let mut result = Vec::with_capacity(contributions.len());
    for contribution in contributions {
        let (id, actions, is_theme) = validate_contribution(extension_id, &contribution)?;
        if !ids.insert(id.clone())
            || actions
                .iter()
                .any(|action| !action_ids.insert(action.clone()))
        {
            return Err(invalid());
        }
        themes = themes
            .checked_add(usize::from(is_theme))
            .ok_or_else(limit)?;
        if themes > super::ui_contract::MAX_THEMES_PER_EXTENSION
            || action_ids.len() > super::ui_contract::MAX_ACTIONS_PER_EXTENSION
        {
            return Err(limit());
        }
        let mut declared_action_ids = actions;
        declared_action_ids.sort_unstable();
        result.push(UiCatalogEntry {
            extension_id: extension_id.to_string(),
            contribution_id: id,
            action_ids: declared_action_ids.clone(),
            declared_action_ids,
            contribution,
        });
    }
    Ok(result)
}

fn authorize(identity: &HostIdentity, extension_id: &str) -> Result<(), String> {
    super::validation::identifier(extension_id)?;
    match identity {
        HostIdentity::Official if extension_id.starts_with("beaver.") => Ok(()),
        HostIdentity::ThirdParty(id) if id == extension_id => Ok(()),
        _ => Err(invalid()),
    }
}

fn validate_contribution(
    owner: &str,
    value: &Value,
) -> Result<(String, Vec<String>, bool), String> {
    let object = value.as_object().ok_or_else(invalid)?;
    let kind = super::ui_view_validation::string(object, "type")?;
    let id = super::ui_view_validation::string(object, "id")?;
    super::ui_view_validation::owned_id(owner, id)?;
    order(object)?;
    if kind == "theme" {
        validate_theme(object)?;
        return Ok((id.to_string(), Vec::new(), true));
    }
    let placement = super::ui_view_validation::string(object, "placement")?;
    let expected = super::ui_contract::UI_PLACEMENTS
        .iter()
        .find(|candidate| candidate.key == placement)
        .ok_or_else(invalid)?
        .contribution_type;
    if expected != kind {
        return Err(invalid());
    }
    super::ui_view_validation::localized(object.get("label").ok_or_else(invalid)?)?;
    if object.get("icon").is_some_and(|icon| {
        !icon
            .as_str()
            .is_some_and(|name| super::ui_contract::UI_ICONS.contains(&name))
    }) {
        return Err(invalid());
    }
    mutation(owner, object)?;
    let mut actions = HashSet::new();
    match kind {
        "action" => {
            exact_common(object, &["actionId"])?;
            let action = super::ui_view_validation::string(object, "actionId")?;
            super::ui_view_validation::owned_id(owner, action)?;
            actions.insert(action.to_string());
        }
        "settingsTab" => {
            exact_common(object, &["detail"])?;
            super::ui_view_validation::validate_view(
                owner,
                object.get("detail").ok_or_else(invalid)?,
                &mut actions,
            )?;
        }
        "tab" => {
            exact_common(object, &["list", "detail"])?;
            if let Some(list) = object.get("list") {
                super::ui_view_validation::validate_view(owner, list, &mut actions)?;
            }
            super::ui_view_validation::validate_view(
                owner,
                object.get("detail").ok_or_else(invalid)?,
                &mut actions,
            )?;
        }
        _ => return Err(invalid()),
    }
    Ok((id.to_string(), actions.into_iter().collect(), false))
}

fn exact_common(object: &Map<String, Value>, extra: &[&str]) -> Result<(), String> {
    let mut allowed = vec!["type", "id", "placement", "order", "label", "icon"];
    allowed.extend(["operation", "targetId"]);
    allowed.extend_from_slice(extra);
    super::ui_view_validation::exact(object, &allowed)
}

fn mutation(owner: &str, object: &Map<String, Value>) -> Result<(), String> {
    match (object.get("operation"), object.get("targetId")) {
        (None, None) => Ok(()),
        (Some(operation), Some(target)) => {
            let operation = operation.as_str().ok_or_else(invalid)?;
            let target = target.as_str().ok_or_else(invalid)?;
            if !super::ui_contract::UI_PLACEMENT_OPERATIONS.contains(&operation) {
                return Err(invalid());
            }
            if target.starts_with("beaver.") {
                super::validation::identifier(target)
            } else {
                super::ui_view_validation::owned_id(owner, target)
            }
        }
        _ => Err(invalid()),
    }
}

fn validate_theme(object: &Map<String, Value>) -> Result<(), String> {
    super::ui_view_validation::exact(object, &["type", "id", "order", "label", "base", "tokens"])?;
    super::ui_view_validation::localized(object.get("label").ok_or_else(invalid)?)?;
    let base = super::ui_view_validation::string(object, "base")?;
    if !super::ui_contract::UI_THEME_BASES.contains(&base) {
        return Err(invalid());
    }
    let tokens = object
        .get("tokens")
        .and_then(Value::as_object)
        .ok_or_else(invalid)?;
    if tokens.len() > super::ui_contract::MAX_THEME_TOKENS {
        return Err(limit());
    }
    for (name, value) in tokens {
        let valid_hex = value.as_str().is_some_and(is_hex_color);
        if !super::ui_contract::UI_THEME_TOKENS.contains(&name.as_str()) || !valid_hex {
            return Err(invalid());
        }
    }
    Ok(())
}

fn order(object: &Map<String, Value>) -> Result<(), String> {
    let value = object
        .get("order")
        .and_then(Value::as_i64)
        .ok_or_else(invalid)?;
    (super::ui_contract::UI_VALIDATION.min_order..=super::ui_contract::UI_VALIDATION.max_order)
        .contains(&value)
        .then_some(())
        .ok_or_else(invalid)
}

fn is_hex_color(value: &str) -> bool {
    matches!(value.len(), 7 | 9)
        && value
            .strip_prefix('#')
            .is_some_and(|content| content.chars().all(|ch| ch.is_ascii_hexdigit()))
}

fn invalid() -> String {
    "ui_contribution_invalid".to_string()
}
fn limit() -> String {
    "ui_limit_exceeded".to_string()
}
