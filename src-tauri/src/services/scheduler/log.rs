use crate::models::{WakeupRun, WakeupRunStatus};
use chrono::{DateTime, Local, Utc};
use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

const MAX_LINES: usize = 500;
const MAX_ID_CHARS: usize = 128;
const MAX_LOG_LINE_BYTES: usize = 2_048;
const MAX_LOG_BYTES: usize = MAX_LINES * MAX_LOG_LINE_BYTES;
static WRITE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn log_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join("wakeups.jsonl")
}

pub async fn log_ok(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    session_id: &str,
    tokens: u32,
) {
    let _ = append(WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Ok,
        error: None,
        session_id: Some(safe_id(session_id)),
        tokens: Some(tokens),
    })
    .await;
}

pub async fn log_err(wakeup_id: &str, scheduled_for: DateTime<Local>, error: &str) {
    let _ = append(error_entry(wakeup_id, scheduled_for, error)).await;
}

pub async fn log_missed(wakeup_id: &str, scheduled_for: DateTime<Local>) {
    let _ = append(WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Missed,
        error: Some("Réveil raté : l'application était indisponible".into()),
        session_id: None,
        tokens: None,
    })
    .await;
}

pub async fn log_cancelled(wakeup_id: &str, scheduled_for: DateTime<Local>) {
    let _ = append(cancelled_entry(wakeup_id, scheduled_for)).await;
}

pub async fn list_runs(wakeup_id: Option<&str>) -> Result<Vec<WakeupRun>, String> {
    let content = match read_bounded_tail(&log_path()).await {
        Ok(content) => content,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(parse_runs(&content, wakeup_id))
}

fn cancelled_entry(wakeup_id: &str, scheduled_for: DateTime<Local>) -> WakeupRun {
    WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Cancelled,
        error: None,
        session_id: None,
        tokens: None,
    }
}

fn error_entry(wakeup_id: &str, scheduled_for: DateTime<Local>, error: &str) -> WakeupRun {
    WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Error,
        error: Some(generic_error(error)),
        session_id: None,
        tokens: None,
    }
}

fn generic_error(error: &str) -> String {
    let lower = error.to_lowercase();
    if lower.contains("rate limit") {
        "Limite de requêtes atteinte".into()
    } else if lower.contains("clé api") || lower.contains("unauthorized") || lower.contains("auth")
    {
        "Authentification échouée".into()
    } else if lower.contains("ollama") {
        "Ollama indisponible".into()
    } else {
        "Le réveil a échoué".into()
    }
}

fn safe_id(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(MAX_ID_CHARS)
        .collect()
}

async fn append(entry: WakeupRun) -> Result<(), String> {
    let _guard = WRITE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;
    let path = log_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| "Erreur journal wakeup".to_string())?;
    }
    let line = format!(
        "{}\n",
        serde_json::to_string(&entry).map_err(|_| "Erreur journal wakeup".to_string())?
    );
    if line.len() > MAX_LOG_LINE_BYTES {
        return Err("Erreur journal wakeup".to_string());
    }
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    file.write_all(line.as_bytes())
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    drop(file);
    trim_if_needed(&path).await
}

fn parse_runs(content: &str, wakeup_id: Option<&str>) -> Vec<WakeupRun> {
    let mut runs = content
        .lines()
        .rev()
        .filter(|line| line.len() <= MAX_LOG_LINE_BYTES)
        .filter_map(|line| serde_json::from_str::<WakeupRun>(line).ok())
        .filter(|run| wakeup_id.map(|id| run.wakeup_id == id).unwrap_or(true))
        .take(MAX_LINES)
        .collect::<Vec<_>>();
    runs.sort_by(|a, b| b.fired_at.cmp(&a.fired_at));
    runs
}

async fn trim_if_needed(path: &PathBuf) -> Result<(), String> {
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    if metadata.len() <= MAX_LOG_BYTES as u64 {
        let content = read_bounded_tail(path).await?;
        if content.lines().count() <= MAX_LINES {
            return Ok(());
        }
    }
    let content = read_bounded_tail(path).await?;
    let mut lines = content
        .lines()
        .rev()
        .filter(|line| line.len() <= MAX_LOG_LINE_BYTES)
        .take(MAX_LINES)
        .collect::<Vec<_>>();
    lines.reverse();
    let trimmed = if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    };
    tokio::fs::write(path, trimmed)
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())
}

async fn read_bounded_tail(path: &PathBuf) -> Result<String, String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    let length = file
        .metadata()
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?
        .len();
    let start = length.saturating_sub(MAX_LOG_BYTES as u64);
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    let mut bytes = Vec::with_capacity((length - start) as usize);
    file.take(MAX_LOG_BYTES as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| "Erreur journal wakeup".to_string())?;
    if start > 0 {
        let first_complete = bytes.iter().position(|byte| *byte == b'\n');
        bytes = first_complete.map_or_else(Vec::new, |index| bytes.split_off(index + 1));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
pub(super) fn cancelled_entry_for_test(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
) -> WakeupRun {
    cancelled_entry(wakeup_id, scheduled_for)
}

#[cfg(test)]
pub(super) fn error_entry_for_test(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    error: &str,
) -> WakeupRun {
    error_entry(wakeup_id, scheduled_for, error)
}

#[cfg(test)]
#[path = "log_tests.rs"]
mod tests;
