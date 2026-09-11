use super::history_metadata::{Metadata, State};
use super::{AutomationError, HistoryEntry, HistoryPage};
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use uuid::Uuid;

pub(crate) const MAX_LINES: usize = 500;
pub(crate) const ROTATED_LINES: usize = 250;
pub(super) const MAX_LINE_BYTES: usize = 2_048;
const MAX_BYTES: usize = MAX_LINES * MAX_LINE_BYTES;
static STATE: OnceLock<tokio::sync::Mutex<State>> = OnceLock::new();

pub(crate) async fn append_at(path: &Path, entry: HistoryEntry) -> Result<(), String> {
    append_inner(
        path,
        entry,
        |path, bytes| async move {
            crate::services::private_store::atomic_write_async(path, bytes).await
        },
        || {},
    )
    .await
    .map_err(|_| unavailable())
}

pub(super) async fn append_inner<Writer, Future, Observer>(
    path: &Path,
    entry: HistoryEntry,
    writer: Writer,
    mut observe_read: Observer,
) -> Result<(), String>
where
    Writer: FnOnce(PathBuf, Vec<u8>) -> Future,
    Future: std::future::Future<Output = Result<(), String>>,
    Observer: FnMut(),
{
    let mut state = state().lock().await;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| unavailable())?;
    }
    let position = match state.position(path) {
        Some(position) if state.get(position).byte_len() == file_len(path).await? => position,
        Some(position) => {
            observe_read();
            *state.get_mut(position) = Metadata::from_content(&read_tail(path).await?);
            position
        }
        None => {
            observe_read();
            let metadata = Metadata::from_content(&read_tail(path).await?);
            state.insert(path.to_path_buf(), metadata)
        }
    };
    if state.get(position).contains(&entry) {
        return Ok(());
    }
    let line = format!(
        "{}\n",
        serde_json::to_string(&entry).map_err(|_| unavailable())?
    );
    if line.len() > MAX_LINE_BYTES {
        return Err(unavailable());
    }
    if state.get(position).needs_rotation(line.len(), MAX_BYTES) {
        observe_read();
        let bytes = rotated(&read_tail(path).await?, &line);
        let metadata = Metadata::from_content(&String::from_utf8_lossy(&bytes));
        writer(path.to_path_buf(), bytes).await?;
        *state.get_mut(position) = metadata;
        return Ok(());
    }
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|_| unavailable())?;
    file.write_all(line.as_bytes())
        .await
        .map_err(|_| unavailable())?;
    file.flush().await.map_err(|_| unavailable())?;
    file.sync_data().await.map_err(|_| unavailable())?;
    state.get_mut(position).record(&entry, line.len());
    Ok(())
}

pub(super) async fn page_at(
    path: &Path,
    automation_id: Uuid,
    limit: Option<usize>,
    cursor: Option<&str>,
) -> Result<HistoryPage, AutomationError> {
    let _guard = state().lock().await;
    let entries = parse(
        &read_tail(path)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?,
    )
    .into_iter()
    .filter(|entry| entry.automation_id == automation_id.to_string())
    .collect::<Vec<_>>();
    let start = match cursor {
        None => 0,
        Some(raw) => {
            let anchor = super::history_cursor::decode(raw)?;
            entries
                .iter()
                .position(|entry| super::history_cursor::matches(entry, &anchor))
                .map(|position| position + 1)
                .ok_or(AutomationError::CursorExpired)?
        }
    };
    let limit = limit.unwrap_or(20).clamp(1, 100);
    let page = entries
        .iter()
        .skip(start)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let next_cursor = (start + page.len() < entries.len())
        .then(|| page.last().map(super::history_cursor::encode))
        .flatten()
        .transpose()?;
    Ok(HistoryPage {
        entries: page,
        next_cursor,
    })
}

pub(crate) async fn all_at(
    path: &Path,
    automation_id: Option<&str>,
) -> Result<Vec<HistoryEntry>, String> {
    let _guard = state().lock().await;
    Ok(parse(&read_tail(path).await?)
        .into_iter()
        .filter(|entry| {
            automation_id
                .map(|id| entry.automation_id == id)
                .unwrap_or(true)
        })
        .collect())
}

pub(crate) fn parse(content: &str) -> Vec<HistoryEntry> {
    let mut entries = content
        .lines()
        .rev()
        .filter(|line| line.len() <= MAX_LINE_BYTES)
        .filter_map(|line| serde_json::from_str::<HistoryEntry>(line).ok())
        .take(MAX_LINES)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| std::cmp::Reverse(history_time(entry)));
    entries
}

fn history_time(entry: &HistoryEntry) -> Option<chrono::DateTime<chrono::FixedOffset>> {
    chrono::DateTime::parse_from_rfc3339(&entry.finished_at).ok()
}

fn rotated(existing: &str, line: &str) -> Vec<u8> {
    let mut lines = existing
        .lines()
        .rev()
        .filter(|line| line.len() <= MAX_LINE_BYTES)
        .take(ROTATED_LINES - 1)
        .collect::<Vec<_>>();
    lines.reverse();
    let mut content = lines.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    content.push_str(line);
    content.into_bytes()
}

async fn file_len(path: &Path) -> Result<usize, String> {
    match tokio::fs::metadata(path).await {
        Ok(metadata) => usize::try_from(metadata.len()).map_err(|_| unavailable()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(_) => Err(unavailable()),
    }
}

async fn read_tail(path: &Path) -> Result<String, String> {
    let mut file = match tokio::fs::File::open(path).await {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(_) => return Err(unavailable()),
    };
    let length = file.metadata().await.map_err(|_| unavailable())?.len();
    let start = length.saturating_sub(MAX_BYTES as u64);
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| unavailable())?;
    let mut bytes = Vec::with_capacity((length - start) as usize);
    file.take(MAX_BYTES as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| unavailable())?;
    if start > 0 {
        bytes = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or_else(Vec::new, |index| bytes.split_off(index + 1));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn state() -> &'static tokio::sync::Mutex<State> {
    STATE.get_or_init(|| tokio::sync::Mutex::new(State::default()))
}

fn unavailable() -> String {
    "wakeup-log-unavailable".into()
}
