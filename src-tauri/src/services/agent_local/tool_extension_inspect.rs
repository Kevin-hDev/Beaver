use serde_json::Value;

use super::types_tools::ToolResult;
use crate::services::extensions::InspectionStatus;

pub async fn execute(args: &Value, session_id: &str, request_id: Option<&str>) -> ToolResult {
    let Ok(ids) = ids(args) else {
        super::tool_extension_catalog_diagnostics::record(session_id, request_id, &[]).await;
        return ToolResult::validation(
            crate::services::extensions::error_codes::INSPECTION_INVALID,
            "Inspection d'extensions invalide.",
        );
    };
    match super::extension_session_state::mutate(session_id, |state| {
        let records = crate::services::extensions::list()?;
        let plugins = crate::services::extensions::indexed_plugins();
        let mut results = Vec::with_capacity(ids.len());
        let mut outcomes = Vec::with_capacity(ids.len());
        for id in &ids {
            let record = records.iter().find(|record| record.manifest.id == *id);
            let plugin = plugins.iter().find(|plugin| plugin.id == *id);
            if !indexed_active_record_is_available(
                record.map(|record| (record.enabled, record.trusted)),
                plugin.is_some(),
            ) {
                // A trusted active record missing from the index indicates an incoherent
                // snapshot. Roll back the whole transaction rather than grant partial access.
                return Err(
                    crate::services::extensions::error_codes::INSPECTION_UNAVAILABLE.to_string(),
                );
            }
            let decision = decision(
                record.map(|record| (record.enabled, record.trusted)),
                plugin.map(|plugin| plugin.tools.is_empty()),
                state.active_plugin_ids.contains(id),
            );
            let mut status = decision.status();
            let Some(plugin) = plugin else {
                outcomes.push((id.clone(), status));
                results.push(serde_json::json!({"id": id, "status": status}));
                continue;
            };
            if decision.admissible() && !discover(state, id) {
                status = InspectionStatus::LimitedByProvider;
            }
            outcomes.push((id.clone(), status));
            results.push(
                serde_json::to_value(crate::services::extensions::inspect_discoverable(
                    plugin, status,
                ))
                .map_err(|_| {
                    crate::services::extensions::error_codes::INSPECTION_UNAVAILABLE.to_string()
                })?,
            );
        }
        Ok((
            crate::services::extensions::serialize_bounded_result(&results).map_err(|_| {
                crate::services::extensions::error_codes::INSPECTION_UNAVAILABLE.to_string()
            })?,
            outcomes,
        ))
    })
    .await
    {
        Ok((content, outcomes)) => {
            super::tool_extension_catalog_diagnostics::record(session_id, request_id, &outcomes).await;
            ToolResult::ok(content)
        }
        Err(_) => {
            super::tool_extension_catalog_diagnostics::record(session_id, request_id, &[]).await;
            ToolResult::unavailable(
                crate::services::extensions::error_codes::INSPECTION_UNAVAILABLE,
                "Extensions indisponibles.",
                true,
            )
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Unknown,
    Inactive,
    Unapproved,
    AlreadyAvailable,
    Loaded,
    NoTools,
}

impl Decision {
    fn status(self) -> InspectionStatus {
        match self {
            Self::Unknown => InspectionStatus::Unknown,
            Self::Inactive => InspectionStatus::Inactive,
            Self::Unapproved => InspectionStatus::Unapproved,
            Self::AlreadyAvailable => InspectionStatus::AlreadyAvailable,
            Self::Loaded => InspectionStatus::Loaded,
            Self::NoTools => InspectionStatus::NoTools,
        }
    }

    fn admissible(self) -> bool {
        // Inspection also grants skills/resources when a plugin's tools were
        // already active in the current provider budget.
        matches!(self, Self::AlreadyAvailable | Self::Loaded | Self::NoTools)
    }
}

fn decision(record: Option<(bool, bool)>, no_tools: Option<bool>, active: bool) -> Decision {
    match record {
        None => Decision::Unknown,
        Some((false, _)) => Decision::Inactive,
        Some((_, false)) => Decision::Unapproved,
        Some(_) if active => Decision::AlreadyAvailable,
        Some(_) if no_tools == Some(true) => Decision::NoTools,
        Some(_) => Decision::Loaded,
    }
}

fn indexed_active_record_is_available(record: Option<(bool, bool)>, indexed: bool) -> bool {
    !matches!(record, Some((true, true))) || indexed
}

fn ids(args: &Value) -> Result<Vec<String>, ()> {
    let values = args.get("ids").and_then(Value::as_array).ok_or(())?;
    if values.is_empty() || values.len() > crate::services::extensions::MAX_INSPECTED_EXTENSIONS {
        return Err(());
    }
    let mut ids = Vec::with_capacity(values.len());
    for value in values {
        let id = value.as_str().ok_or(())?;
        if crate::services::extensions::validate_identifier(id).is_err()
            || ids.iter().any(|item| item == id)
        {
            return Err(());
        }
        ids.push(id.to_string());
    }
    Ok(ids)
}

fn discover(state: &mut super::extension_session_state::ExtensionSessionState, id: &str) -> bool {
    discover_with_refresh(state, id, |state| {
        super::extension_session_plugins::refresh_active(state, false)
    })
}

fn discover_with_refresh(
    state: &mut super::extension_session_state::ExtensionSessionState,
    id: &str,
    refresh: impl Fn(&mut super::extension_session_state::ExtensionSessionState),
) -> bool {
    let mut proposed = state.discovered_plugin_ids.clone();
    if !proposed.contains(&id.to_string()) {
        if proposed.len() >= crate::services::extensions::MAX_DISCOVERED_PLUGINS {
            return false;
        }
        proposed.push(id.to_string());
    }
    let previous = std::mem::replace(&mut state.discovered_plugin_ids, proposed);
    refresh(state);
    if state.active_plugin_ids.contains(&id.to_string()) {
        true
    } else {
        state.discovered_plugin_ids = previous;
        refresh(state);
        false
    }
}

#[cfg(test)]
#[path = "tool_extension_inspect_tests.rs"]
mod tests;
