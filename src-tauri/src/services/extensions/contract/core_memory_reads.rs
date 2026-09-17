use crate::services::agent_local::memory_paths::MemoryScope;
use serde::Serialize;
use serde_json::{json, Value};

use super::core_bridge::{CoreResponse, ExtensionBridgeError};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Summary {
    id: String,
    title: String,
    updated_at: String,
}

pub(super) async fn list(
    context: &super::call_context::ExtensionCallContext,
    params: &Value,
    scope: &MemoryScope,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let session = session_id(context)?;
    require_read(session)?;
    if !scope.root.exists() {
        return Ok(CoreResponse::Json(json!({"items": [], "nextCursor": null})));
    }
    let offset = cursor(params)?;
    let mut topics = crate::services::agent_local::memory_store::list_topics(scope).await;
    topics.sort_by(|left, right| {
        right
            .topic
            .updated_at
            .cmp(&left.topic.updated_at)
            .then_with(|| left.topic.id.cmp(&right.topic.id))
    });
    let total = topics.len();
    let mut items = Vec::new();
    for parsed in topics
        .into_iter()
        .skip(offset)
        .take(super::types::MAX_SDK_PAGE_RESULTS)
    {
        let summary = Summary {
            id: parsed.topic.id,
            title: parsed.topic.title,
            updated_at: parsed.topic.updated_at,
        };
        let encoded = serde_json::to_string(&summary).map_err(|_| ExtensionBridgeError::Failed)?;
        let (_, truncated) =
            crate::services::agent_local::memory_runtime::consume_result(session, &encoded);
        if truncated {
            break;
        }
        items.push(summary);
    }
    let next =
        (offset.saturating_add(items.len()) < total).then(|| (offset + items.len()).to_string());
    Ok(CoreResponse::Json(
        json!({"items": items, "nextCursor": next}),
    ))
}

pub(super) async fn read(
    context: &super::call_context::ExtensionCallContext,
    params: &Value,
    scope: &MemoryScope,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let session = session_id(context)?;
    require_read(session)?;
    let path = topic_path(scope, params)?;
    let (parsed, content) = crate::services::agent_local::memory_store::read_topic(scope, &path)
        .await
        .map_err(map_read_error)?;
    let (content, _) =
        crate::services::agent_local::memory_runtime::consume_result(session, &content);
    Ok(CoreResponse::Json(topic_value(&parsed.topic, content)))
}

pub(super) fn topic_value(
    topic: &crate::services::agent_local::memory_types::MemoryTopic,
    content: String,
) -> Value {
    json!({
        "id": topic.id,
        "title": topic.title,
        "updatedAt": topic.updated_at,
        "content": content,
    })
}

pub(super) fn topic_path(
    scope: &MemoryScope,
    params: &Value,
) -> Result<std::path::PathBuf, ExtensionBridgeError> {
    let id = params
        .get("topicId")
        .and_then(Value::as_str)
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .ok_or(ExtensionBridgeError::Denied)?;
    Ok(scope.topics_dir().join(format!("{id}.md")))
}

pub(super) fn session_id(
    context: &super::call_context::ExtensionCallContext,
) -> Result<&str, ExtensionBridgeError> {
    context
        .core_scope()
        .map(|scope| scope.agent.session_id.as_str())
        .filter(|value| !value.is_empty())
        .ok_or(ExtensionBridgeError::Denied)
}

pub(super) fn map_read_error(
    error: crate::services::agent_local::memory_store::MemoryEditError,
) -> ExtensionBridgeError {
    match error {
        crate::services::agent_local::memory_store::MemoryEditError::NotFound => {
            ExtensionBridgeError::Backend("core_memory_not_found")
        }
        crate::services::agent_local::memory_store::MemoryEditError::Stale => {
            ExtensionBridgeError::Backend("core_memory_stale")
        }
        crate::services::agent_local::memory_store::MemoryEditError::Failed(_) => {
            ExtensionBridgeError::Failed
        }
    }
}

fn require_read(session: &str) -> Result<(), ExtensionBridgeError> {
    crate::services::agent_local::memory_runtime::read_allowed(session)
        .then_some(())
        .ok_or(ExtensionBridgeError::Denied)
}

fn cursor(params: &Value) -> Result<usize, ExtensionBridgeError> {
    match params.get("cursor") {
        None | Some(Value::Null) => Ok(0),
        Some(Value::String(value)) => value.parse().map_err(|_| ExtensionBridgeError::Denied),
        _ => Err(ExtensionBridgeError::Denied),
    }
}
