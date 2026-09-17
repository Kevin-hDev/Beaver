use serde::Serialize;
use serde_json::{json, Value};
use std::time::Duration;

use super::core_bridge::{CoreResponse, ExtensionBridgeError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Descriptor {
    connection_id: String,
    canonical_provider: String,
    transport_family: String,
    model_id: String,
    name: String,
    generation_supported: bool,
}

pub(super) async fn list(params: &Value) -> Result<CoreResponse, ExtensionBridgeError> {
    let requested = optional_string(params, "connectionId")?;
    let mut connections = requested.map_or_else(configured_connections, |value| vec![value]);
    connections.sort();
    connections.dedup();
    let mut items = Vec::new();
    let mut incomplete = false;
    for connection in connections {
        if !is_configured(&connection) {
            continue;
        }
        let Some(route) =
            crate::services::llm::stream_dispatch::model_route_descriptor(&connection)
        else {
            continue;
        };
        match models_for(&connection).await {
            Some(models) => items.extend(
                models
                    .into_iter()
                    .map(|model| descriptor(&connection, &route, model)),
            ),
            None => incomplete = true,
        }
    }
    items.sort_by(|left, right| {
        (&left.connection_id, &left.model_id).cmp(&(&right.connection_id, &right.model_id))
    });
    let offset = cursor(params)?;
    let total = items.len();
    let page = items
        .into_iter()
        .skip(offset)
        .take(super::types::MAX_SDK_PAGE_RESULTS)
        .collect::<Vec<_>>();
    let next =
        (offset.saturating_add(page.len()) < total).then(|| (offset + page.len()).to_string());
    Ok(CoreResponse::Json(json!({
        "items": page,
        "nextCursor": next,
        "incomplete": incomplete,
    })))
}

fn descriptor(
    connection: &str,
    route: &crate::services::llm::stream_dispatch::ModelRouteDescriptor,
    model: crate::services::llm::types::ModelInfo,
) -> Descriptor {
    Descriptor {
        connection_id: connection.to_string(),
        canonical_provider: route.canonical_provider.to_string(),
        transport_family: route.transport_family.to_string(),
        name: model.display_name.unwrap_or_else(|| model.id.clone()),
        model_id: model.id,
        generation_supported: route.generation_supported,
    }
}

pub(super) fn is_configured(connection: &str) -> bool {
    connection == "ollama"
        || crate::services::api_keys::has_key(connection)
        || crate::services::oauth_providers::list_statuses()
            .into_iter()
            .any(|status| status.connected && status.connection_id == connection)
}

pub(super) async fn known_model(connection: &str, model: &str) -> bool {
    models_for(connection)
        .await
        .is_some_and(|models| models.iter().any(|candidate| candidate.id == model))
}

async fn models_for(connection: &str) -> Option<Vec<crate::services::llm::types::ModelInfo>> {
    if connection == "codex-oauth" {
        return crate::services::codex_client::model_catalog::cached_models().await;
    }
    if connection == "ollama" {
        return crate::services::agent_local::ollama_collect::list_extension_models(
            Duration::from_millis(super::types::MODEL_GENERATION_TIMEOUT_MS as u64),
        )
        .await
        .ok()
        .map(|models| models.into_iter().map(minimal_model).collect());
    }
    crate::services::llm::runtime_models::snapshot(connection)
}

fn configured_connections() -> Vec<String> {
    let mut values = crate::services::api_keys::list_configured();
    values.extend(
        crate::services::oauth_providers::list_statuses()
            .into_iter()
            .filter(|status| status.connected)
            .map(|status| status.connection_id.to_string()),
    );
    values.push("ollama".to_string());
    values
}

fn minimal_model(id: String) -> crate::services::llm::types::ModelInfo {
    crate::services::llm::types::ModelInfo {
        id,
        display_name: None,
        owned_by: None,
        context_length: None,
        max_output_tokens: None,
        supported_parameters: None,
        catalog_capabilities: Default::default(),
        supports_tools: false,
        supports_vision: false,
        supports_thinking: false,
        reasoning_contract: None,
        supports_fast_mode: false,
        context_usage_includes_reasoning: true,
        is_free: true,
    }
}

fn optional_string(params: &Value, key: &str) -> Result<Option<String>, ExtensionBridgeError> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value.clone())),
        _ => Err(ExtensionBridgeError::Denied),
    }
}

fn cursor(params: &Value) -> Result<usize, ExtensionBridgeError> {
    optional_string(params, "cursor")?
        .map(|value| value.parse().map_err(|_| ExtensionBridgeError::Denied))
        .transpose()
        .map(|value| value.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_listing_does_not_fetch_or_expose_secrets() {
        let route = crate::services::llm::stream_dispatch::model_route_descriptor("openai")
            .expect("OpenAI route");
        let value = serde_json::to_value(descriptor("openai", &route, minimal_model("gpt".into())))
            .unwrap();
        assert_eq!(value["connectionId"], "openai");
        for forbidden in ["baseUrl", "workspaceId", "headers", "apiKey", "endpoint"] {
            assert!(value.get(forbidden).is_none(), "{forbidden}");
        }
    }

    #[test]
    fn unknown_connection_never_creates_a_transport() {
        assert!(crate::services::llm::stream_dispatch::model_route_descriptor("unknown").is_none());
        assert!(!is_configured("unknown"));
    }
}
