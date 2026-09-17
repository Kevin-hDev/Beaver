use crate::services::agent_local::memory_paths::MemoryScope;
use crate::services::agent_local::memory_store::{MemoryEditError, MemoryWriteError};
use serde_json::{json, Value};

use super::core_bridge::{CoreResponse, ExtensionBridgeError};

pub(super) async fn write(
    context: &super::call_context::ExtensionCallContext,
    params: &Value,
    scope: &MemoryScope,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let session = super::core_memory_reads::session_id(context)?;
    let defaults = crate::services::agent_local::memory_runtime::write_defaults(session)
        .ok_or(ExtensionBridgeError::Denied)?;
    let input = required(params, "content")?;
    let (path, content, index_updated) = match optional(params, "topicId")? {
        Some(id) => {
            uuid::Uuid::parse_str(id).map_err(|_| ExtensionBridgeError::Denied)?;
            let expected = required(params, "expectedUpdatedAt")?;
            let path = scope.topics_dir().join(format!("{id}.md"));
            let (current, _) = crate::services::agent_local::memory_store::read_topic(scope, &path)
                .await
                .map_err(super::core_memory_reads::map_read_error)?;
            let content = super::core_memory_content::for_update(
                input,
                &current.topic,
                crate::services::agent_local::memory_store::scope_kind(scope),
            )
            .map_err(|_| ExtensionBridgeError::Denied)?;
            let index_updated = mutation_result(
                crate::services::agent_local::memory_store::replace_topic(
                    scope, &path, expected, &content,
                )
                .await,
            )?;
            (path, content, index_updated)
        }
        None => {
            if params.get("expectedUpdatedAt").is_some() {
                return Err(ExtensionBridgeError::Denied);
            }
            let id = uuid::Uuid::new_v4().to_string();
            let path = scope.topics_dir().join(format!("{id}.md"));
            let content = super::core_memory_content::for_create(
                input,
                &id,
                crate::services::agent_local::memory_store::scope_kind(scope),
                session,
                defaults.0,
                defaults.1,
            )
            .map_err(|_| ExtensionBridgeError::Denied)?;
            let index_updated = write_result(
                crate::services::agent_local::memory_store::write_topic(scope, &path, &content)
                    .await,
            )?;
            (path, content, index_updated)
        }
    };
    mutation_response(session, scope, &path, content, index_updated)
}

pub(super) async fn archive(
    context: &super::call_context::ExtensionCallContext,
    params: &Value,
    scope: &MemoryScope,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let session = super::core_memory_reads::session_id(context)?;
    crate::services::agent_local::memory_runtime::write_allowed(session)
        .then_some(())
        .ok_or(ExtensionBridgeError::Denied)?;
    let path = super::core_memory_reads::topic_path(scope, params)?;
    crate::services::agent_local::memory_store::read_topic(scope, &path)
        .await
        .map_err(super::core_memory_reads::map_read_error)?;
    let outcome = crate::services::agent_local::memory_store::archive_topic_result(scope, &path)
        .await
        .map_err(map_write_error)?;
    mutation_response(
        session,
        scope,
        &path,
        outcome.content,
        outcome.index_updated,
    )
}

fn mutation_response(
    session: &str,
    scope: &MemoryScope,
    path: &std::path::Path,
    content: String,
    index_updated: bool,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let parsed = crate::services::agent_local::memory_format::parse(
        &content,
        path,
        crate::services::agent_local::memory_store::scope_kind(scope),
    )
    .map_err(|_| ExtensionBridgeError::Failed)?;
    let (content, _) =
        crate::services::agent_local::memory_runtime::consume_result(session, &content);
    Ok(CoreResponse::Json(json!({
        "topic": super::core_memory_reads::topic_value(&parsed.topic, content),
        "applied": true,
        "indexUpdated": index_updated,
    })))
}

fn mutation_result(
    result: Result<Vec<String>, MemoryEditError>,
) -> Result<bool, ExtensionBridgeError> {
    match result {
        Ok(_) => Ok(true),
        Err(MemoryEditError::Stale) => Err(ExtensionBridgeError::Backend("core_memory_stale")),
        Err(MemoryEditError::NotFound) => {
            Err(ExtensionBridgeError::Backend("core_memory_not_found"))
        }
        Err(MemoryEditError::Failed(error)) => write_result(Err(error)),
    }
}

fn write_result(
    result: Result<Vec<String>, MemoryWriteError>,
) -> Result<bool, ExtensionBridgeError> {
    match result {
        Ok(_) => Ok(true),
        Err(MemoryWriteError::AppliedButIndexFailed(_)) => Ok(false),
        Err(error) => Err(map_write_error(error)),
    }
}

fn map_write_error(error: MemoryWriteError) -> ExtensionBridgeError {
    match error {
        MemoryWriteError::TargetInvalid(_) | MemoryWriteError::ContentInvalid(_) => {
            ExtensionBridgeError::Denied
        }
        MemoryWriteError::AppliedButIndexFailed(_) => {
            ExtensionBridgeError::Backend("core_partial_apply")
        }
        MemoryWriteError::SetupUnavailable(_)
        | MemoryWriteError::LimitReached
        | MemoryWriteError::SourceUnavailable(_)
        | MemoryWriteError::StorageFailed(_) => ExtensionBridgeError::Failed,
    }
}

fn required<'a>(params: &'a Value, key: &str) -> Result<&'a str, ExtensionBridgeError> {
    optional(params, key)?.ok_or(ExtensionBridgeError::Denied)
}

fn optional<'a>(params: &'a Value, key: &str) -> Result<Option<&'a str>, ExtensionBridgeError> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value)),
        _ => Err(ExtensionBridgeError::Denied),
    }
}
