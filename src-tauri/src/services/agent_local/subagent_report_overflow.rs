use std::sync::LazyLock;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::types_session::{AgentSession, SubagentHiddenReport};

const FILE_NAME: &str = "subagent-report-overflow.json";
const MAX_PENDING: usize = 64;
const MAX_BYTES: u64 = 1024 * 1024;
const SCHEMA_VERSION: u16 = 1;
static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u16,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    parent_session_id: String,
    report: SubagentHiddenReport,
}

pub async fn enqueue(parent_session_id: &str, report: SubagentHiddenReport) -> Result<(), String> {
    super::session_store::validate_session_id(parent_session_id)?;
    let _guard = LOCK.lock().await;
    let mut document = load().await?;
    if document.entries.iter().any(|entry| entry.report.id == report.id) {
        return Ok(());
    }
    if document.entries.len() >= MAX_PENDING {
        return Err("subagent_report_overflow_full".into());
    }
    document.entries.push(Entry {
        parent_session_id: parent_session_id.to_string(),
        report,
    });
    save(&document).await
}

pub async fn pending_for_parent(parent_session_id: &str) -> Vec<SubagentHiddenReport> {
    let _guard = LOCK.lock().await;
    load()
        .await
        .map(|document| {
            document
                .entries
                .into_iter()
                .filter(|entry| entry.parent_session_id == parent_session_id)
                .map(|entry| entry.report)
                .collect()
        })
        .unwrap_or_default()
}

pub async fn drain_into_parent(session: &mut AgentSession) -> Result<(), String> {
    let _guard = LOCK.lock().await;
    let mut document = load().await?;
    let candidates = document
        .entries
        .iter()
        .filter(|entry| entry.parent_session_id == session.id)
        .cloned()
        .collect::<Vec<_>>();
    let mut persisted = Vec::new();
    for entry in candidates {
        if super::subagent_hidden_reports::append_locked(session, entry.report.clone())
            .await
            .is_err()
        {
            break;
        }
        persisted.push(entry.report.id);
    }
    if persisted.is_empty() {
        return Ok(());
    }
    document
        .entries
        .retain(|entry| !persisted.contains(&entry.report.id));
    save(&document).await
}

pub async fn remove_for_parent(parent_session_id: &str) -> Result<(), String> {
    let _guard = LOCK.lock().await;
    let mut document = load().await?;
    let before = document.entries.len();
    document
        .entries
        .retain(|entry| entry.parent_session_id != parent_session_id);
    if document.entries.len() == before {
        return Ok(());
    }
    save(&document).await
}

async fn load() -> Result<Document, String> {
    let path = crate::services::paths::data_dir()
        .join("subagent-reports")
        .join(FILE_NAME);
    let bytes = match crate::services::private_store::read_bounded_regular_async(path, MAX_BYTES)
        .await?
    {
        crate::services::private_store::BoundedFile::Missing => {
            return Ok(Document {
                schema_version: SCHEMA_VERSION,
                entries: Vec::new(),
            });
        }
        crate::services::private_store::BoundedFile::Content(bytes) => bytes,
    };
    let document: Document = serde_json::from_slice(&bytes)
        .map_err(|_| "subagent_report_overflow_invalid".to_string())?;
    if document.schema_version != SCHEMA_VERSION || document.entries.len() > MAX_PENDING {
        return Err("subagent_report_overflow_invalid".into());
    }
    Ok(document)
}

async fn save(document: &Document) -> Result<(), String> {
    let bytes = serde_json::to_vec(document)
        .map_err(|_| "subagent_report_overflow_unavailable".to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("subagent_report_overflow_full".into());
    }
    let path = crate::services::paths::data_file_for_write("subagent-reports", FILE_NAME)
        .await
        .map_err(|_| "subagent_report_overflow_unavailable".to_string())?;
    crate::services::private_store::atomic_write_async(path, bytes).await
}
