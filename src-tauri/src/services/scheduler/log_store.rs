use crate::models::WakeupRun;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

pub(super) const MAX_LINES: usize = 500;
pub(super) const MAX_ID_CHARS: usize = 128;
pub(super) const MAX_LOG_LINE_BYTES: usize = 2_048;
const MAX_LOG_BYTES: usize = MAX_LINES * MAX_LOG_LINE_BYTES;
static LOG_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

pub(super) async fn append_at(path: &Path, entry: WakeupRun) -> Result<(), String> {
    append_at_inner(path, entry, |path, bytes| async move {
        crate::services::private_store::atomic_write_async(path, bytes).await
    })
    .await
    .map_err(|_| log_error())
}

#[cfg(test)]
pub(super) async fn append_at_with_atomic_writer<Writer, WriteFuture>(
    path: &Path,
    entry: WakeupRun,
    atomic_writer: Writer,
) -> Result<(), String>
where
    Writer: FnOnce(PathBuf, Vec<u8>) -> WriteFuture,
    WriteFuture: std::future::Future<Output = Result<(), String>>,
{
    append_at_inner(path, entry, atomic_writer).await
}

async fn append_at_inner<Writer, WriteFuture>(
    path: &Path,
    entry: WakeupRun,
    atomic_writer: Writer,
) -> Result<(), String>
where
    Writer: FnOnce(PathBuf, Vec<u8>) -> WriteFuture,
    WriteFuture: std::future::Future<Output = Result<(), String>>,
{
    let _guard = log_lock().lock().await;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| log_error())?;
    }
    let existing = read_bounded_tail(path).await?;
    if contains_occurrence(&existing, &entry) {
        return Ok(());
    }
    let line = format!(
        "{}\n",
        serde_json::to_string(&entry).map_err(|_| log_error())?
    );
    if line.len() > MAX_LOG_LINE_BYTES {
        return Err(log_error());
    }
    if needs_rotation(&existing, line.len()) {
        return atomic_writer(path.to_path_buf(), rotated_content(&existing, &line)).await;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|_| log_error())?;
    file.write_all(line.as_bytes())
        .await
        .map_err(|_| log_error())?;
    file.flush().await.map_err(|_| log_error())?;
    file.sync_data().await.map_err(|_| log_error())?;
    Ok(())
}

pub(super) async fn list_runs_at(
    path: &Path,
    wakeup_id: Option<&str>,
) -> Result<Vec<WakeupRun>, String> {
    let _guard = log_lock().lock().await;
    Ok(parse_runs(&read_bounded_tail(path).await?, wakeup_id))
}

fn contains_occurrence(content: &str, entry: &WakeupRun) -> bool {
    content
        .lines()
        .filter(|line| line.len() <= MAX_LOG_LINE_BYTES)
        .filter_map(|line| serde_json::from_str::<WakeupRun>(line).ok())
        .any(|run| run.wakeup_id == entry.wakeup_id && run.scheduled_for == entry.scheduled_for)
}

pub(super) fn parse_runs(content: &str, wakeup_id: Option<&str>) -> Vec<WakeupRun> {
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

fn needs_rotation(existing: &str, new_line_bytes: usize) -> bool {
    existing.lines().count() >= MAX_LINES
        || existing.len().saturating_add(new_line_bytes) > MAX_LOG_BYTES
}

fn rotated_content(existing: &str, new_line: &str) -> Vec<u8> {
    let mut lines = existing
        .lines()
        .rev()
        .filter(|line| line.len() <= MAX_LOG_LINE_BYTES)
        .take(MAX_LINES - 1)
        .collect::<Vec<_>>();
    lines.reverse();
    let mut rotated = lines.join("\n");
    if !rotated.is_empty() {
        rotated.push('\n');
    }
    rotated.push_str(new_line);
    rotated.into_bytes()
}

async fn read_bounded_tail(path: &Path) -> Result<String, String> {
    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(_) => return Err(log_error()),
    };
    let length = file.metadata().await.map_err(|_| log_error())?.len();
    let start = length.saturating_sub(MAX_LOG_BYTES as u64);
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| log_error())?;
    let mut bytes = Vec::with_capacity((length - start) as usize);
    file.take(MAX_LOG_BYTES as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| log_error())?;
    if start > 0 {
        bytes = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or_else(Vec::new, |index| bytes.split_off(index + 1));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn log_lock() -> &'static tokio::sync::Mutex<()> {
    LOG_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn log_error() -> String {
    "wakeup-log-unavailable".to_string()
}
