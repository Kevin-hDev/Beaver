use super::{AutomationActor, AutomationError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use uuid::Uuid;

pub(super) const MAX_LINES: usize = 500;
pub(super) const ROTATED_LINES: usize = 250;
const MAX_BYTES: u64 = 1024 * 1024;
static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AuditEntry {
    pub operation_id: Uuid,
    pub action: String,
    pub origin: super::AutomationOrigin,
    pub actor_id: String,
    pub target_id: Option<Uuid>,
    pub result: String,
    pub fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    pub at: DateTime<Utc>,
}

pub(super) async fn begin(
    root: &Path,
    actor: &AutomationActor,
    action: &str,
    target_id: Option<Uuid>,
    fields: Vec<String>,
) -> Result<Uuid, AutomationError> {
    let operation_id = Uuid::new_v4();
    append(
        root,
        AuditEntry {
            operation_id,
            action: action.to_string(),
            origin: actor.origin,
            actor_id: safe_actor_id(&actor.session_or_channel_id)?,
            target_id,
            result: "intent".into(),
            fields,
            count: None,
            at: Utc::now(),
        },
    )
    .await?;
    Ok(operation_id)
}

pub(super) async fn complete<T>(
    root: &Path,
    actor: &AutomationActor,
    operation_id: Uuid,
    action: &str,
    target_id: Option<Uuid>,
    result: Result<T, AutomationError>,
    count: Option<usize>,
) -> Result<T, AutomationError> {
    let result_name = result.as_ref().map_or_else(|error| error.code(), |_| "ok");
    append(
        root,
        AuditEntry {
            operation_id,
            action: action.to_string(),
            origin: actor.origin,
            actor_id: safe_actor_id(&actor.session_or_channel_id)?,
            target_id,
            result: result_name.to_string(),
            fields: Vec::new(),
            count,
            at: Utc::now(),
        },
    )
    .await?;
    result
}

async fn append(root: &Path, entry: AuditEntry) -> Result<(), AutomationError> {
    let _guard = LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;
    let path = path(root);
    let content =
        match crate::services::private_store::read_bounded_regular_async(path.clone(), MAX_BYTES)
            .await
            .map_err(|_| AutomationError::AuditUnavailable)?
        {
            crate::services::private_store::BoundedFile::Missing => String::new(),
            crate::services::private_store::BoundedFile::Content(bytes) => {
                String::from_utf8(bytes).map_err(|_| AutomationError::AuditUnavailable)?
            }
        };
    let mut lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    if lines.len() >= MAX_LINES {
        lines = lines.split_off(lines.len().saturating_sub(ROTATED_LINES - 1));
    }
    lines.push(serde_json::to_string(&entry).map_err(|_| AutomationError::AuditUnavailable)?);
    let mut bytes = lines.join("\n").into_bytes();
    bytes.push(b'\n');
    crate::services::private_store::atomic_write_async(path, bytes)
        .await
        .map_err(|_| AutomationError::AuditUnavailable)
}

fn safe_actor_id(value: &str) -> Result<String, AutomationError> {
    if value.is_empty() || value.chars().count() > 128 || value.chars().any(char::is_control) {
        return Err(AutomationError::AuditUnavailable);
    }
    Ok(value.to_string())
}

fn path(root: &Path) -> PathBuf {
    root.join("logs").join("automation-audit.jsonl")
}
