use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use super::stream_recovery_record::{validate_header, StreamRecoveryHeader, StreamRecoveryRecord};

pub(crate) const MAX_LOG_BYTES: u64 = 32 * 1024 * 1024;
pub(crate) const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_LOGS_PER_SESSION: usize = 4;
pub(crate) const MAX_DISCOVERY_ENTRIES: usize = 4_096;
static STORE_GATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(crate) async fn create(header: &StreamRecoveryHeader) -> Result<(PathBuf, File), String> {
    validate_header(header)?;
    let _gate = STORE_GATE.lock().await;
    let path = path_for(&header.session_id, &header.request_id)?;
    ensure_capacity(&header.session_id).await?;
    let mut bytes = serde_json::to_vec(&StreamRecoveryRecord::Header(header.clone()))
        .map_err(|_| error())?;
    bytes.push(b'\n');
    crate::services::private_store::write_new_async(path.clone(), bytes).await?;
    let file = open_append(&path)?;
    Ok((path, file))
}

pub(crate) fn open_append(path: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.append(true).read(true);
    crate::services::private_store::configure_open_no_follow(&mut options);
    let file = options.open(path).map_err(|_| error())?;
    crate::services::private_store::file_is_single_link_regular(&file)
        .then_some(file)
        .ok_or_else(error)
}

pub(crate) fn path_for(session_id: &str, request_id: &str) -> Result<PathBuf, String> {
    super::session_store::validate_session_id(session_id)?;
    uuid::Uuid::parse_str(request_id).map_err(|_| error())?;
    Ok(root().join(session_id).join(format!("{request_id}.jsonl")))
}

pub(crate) fn root() -> PathBuf {
    crate::services::paths::data_dir().join("agent-stream-recovery")
}

pub(crate) async fn remove(path: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|_| error())?;
        }
        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
        Ok(())
    })
    .await
    .map_err(|_| error())?
}

pub(crate) fn visit_records(
    path: &Path,
    mut visit: impl FnMut(StreamRecoveryRecord) -> Result<(), String>,
) -> Result<(), String> {
    let file = crate::services::private_store::open_regular_single_link(path)?
        .ok_or_else(error)?;
    if file.metadata().map_err(|_| error())?.len() > MAX_LOG_BYTES {
        return Err(error());
    }
    let mut reader = BufReader::new(file);
    let mut previous = None;
    let mut first = true;
    while let Some((line, terminated)) = read_line(&mut reader)? {
        if !terminated {
            break;
        }
        let record: StreamRecoveryRecord = serde_json::from_slice(&line).map_err(|_| error())?;
        match &record {
            StreamRecoveryRecord::Header(header) if first => validate_header(header)?,
            StreamRecoveryRecord::Header(_) => return Err(error()),
            _ if first => return Err(error()),
            other => {
                let sequence = sequence(other);
                if previous.is_some_and(|value| sequence <= value) {
                    return Err(error());
                }
                previous = Some(sequence);
            }
        }
        first = false;
        visit(record)?;
    }
    (!first).then_some(()).ok_or_else(error)
}

fn read_line(reader: &mut impl BufRead) -> Result<Option<(Vec<u8>, bool)>, String> {
    let mut line = Vec::new();
    loop {
        let buffer = reader.fill_buf().map_err(|_| error())?;
        if buffer.is_empty() {
            return Ok((!line.is_empty()).then_some((line, false)));
        }
        let take = buffer
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(buffer.len(), |index| index + 1);
        if line.len().saturating_add(take) > MAX_LINE_BYTES {
            return Err(error());
        }
        line.extend_from_slice(&buffer[..take]);
        reader.consume(take);
        if line.last() == Some(&b'\n') {
            line.pop();
            return Ok(Some((line, true)));
        }
    }
}

fn sequence(record: &StreamRecoveryRecord) -> u64 {
    match record {
        StreamRecoveryRecord::Event { sequence, .. }
        | StreamRecoveryRecord::PendingMessage { sequence, .. }
        | StreamRecoveryRecord::TurnReady { sequence } => *sequence,
        StreamRecoveryRecord::Header(_) => 0,
    }
}

async fn ensure_capacity(session_id: &str) -> Result<(), String> {
    let root = root();
    let session = session_id.to_string();
    tokio::task::spawn_blocking(move || {
        let mut total = 0usize;
        let mut own = 0usize;
        if root.exists() {
            let mut inspected = 0usize;
            for directory in std::fs::read_dir(&root).map_err(|_| error())? {
                if inspected >= MAX_DISCOVERY_ENTRIES {
                    return Err(error());
                }
                inspected += 1;
                let directory = directory.map_err(|_| error())?;
                if !directory.file_type().map_err(|_| error())?.is_dir() {
                    continue;
                }
                let is_own = directory.file_name().to_string_lossy() == session;
                for entry in std::fs::read_dir(directory.path()).map_err(|_| error())? {
                    if inspected >= MAX_DISCOVERY_ENTRIES {
                        return Err(error());
                    }
                    inspected += 1;
                    let entry = entry.map_err(|_| error())?;
                    if entry.file_type().map_err(|_| error())?.is_file() {
                        total += 1;
                        own += usize::from(is_own);
                    }
                }
            }
        }
        if total >= super::stream_recovery_owners::MAX_STREAM_RECOVERY_LOGS
            || own >= MAX_LOGS_PER_SESSION
        {
            return Err(error());
        }
        Ok(())
    })
    .await
    .map_err(|_| error())?
}

fn error() -> String {
    "stream_recovery_unavailable".to_string()
}
