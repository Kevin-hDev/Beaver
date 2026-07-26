use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::{LazyLock, Mutex};

const MAX_ENTRIES: usize = 10_000;
const MAX_LINE_BYTES: usize = 2_048;

#[derive(Default)]
struct WriterState {
    path: Option<PathBuf>,
    known_lines: Option<usize>,
    last_trim_day: Option<i64>,
}

static WRITER: LazyLock<Mutex<WriterState>> =
    LazyLock::new(|| Mutex::new(WriterState::default()));

fn append_serialized(path: &Path, line: &str, retention_days: u32) -> Result<(), String> {
    if line.is_empty() || line.len() > MAX_LINE_BYTES || line.contains('\n') {
        return Err("journal d'audit invalide".to_string());
    }
    let mut state = WRITER
        .lock()
        .map_err(|_| "journal d'audit indisponible".to_string())?;
    if state.path.as_deref() != Some(path) {
        *state = WriterState {
            path: Some(path.to_path_buf()),
            ..WriterState::default()
        };
    }
    let now = chrono::Utc::now();
    let day = now.timestamp().div_euclid(86_400);
    if state.known_lines.is_none()
        || state.last_trim_day != Some(day)
        || state.known_lines.is_some_and(|count| count >= MAX_ENTRIES)
    {
        state.known_lines = Some(trim(path, retention_days, now.timestamp())?);
        state.last_trim_day = Some(day);
    }
    ensure_private_file(path)?;
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|_| "journal d'audit indisponible".to_string())?;
    let mut bytes = Vec::with_capacity(line.len() + 1);
    bytes.extend_from_slice(line.as_bytes());
    bytes.push(b'\n');
    file.write_all(&bytes)
        .and_then(|_| file.sync_data())
        .map_err(|_| "journal d'audit indisponible".to_string())?;
    state.known_lines = Some(state.known_lines.unwrap_or(0).saturating_add(1));
    Ok(())
}

fn ensure_private_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        crate::services::private_store::repair_path(path)
    } else {
        crate::services::private_store::atomic_write(path, b"")
    }
}

fn trim(path: &Path, retention_days: u32, now: i64) -> Result<usize, String> {
    if !path.exists() {
        crate::services::private_store::atomic_write(path, b"")?;
        return Ok(0);
    }
    let cutoff = now.saturating_sub(i64::from(retention_days.clamp(1, 365)) * 86_400);
    let mut reader = BufReader::new(
        File::open(path).map_err(|_| "journal d'audit indisponible".to_string())?,
    );
    let mut kept = VecDeque::with_capacity(MAX_ENTRIES);
    while let Some(line) = read_bounded_line(&mut reader)? {
        let Ok(entry) = serde_json::from_slice::<AuditEntry>(&line) else {
            continue;
        };
        let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) else {
            continue;
        };
        if timestamp.timestamp() < cutoff {
            continue;
        }
        if kept.len() >= MAX_ENTRIES - 1 {
            kept.pop_front();
        }
        kept.push_back(line);
    }
    let mut output = Vec::with_capacity(kept.len().saturating_mul(256));
    for line in &kept {
        output.extend_from_slice(line);
        output.push(b'\n');
    }
    crate::services::private_store::atomic_write(path, &output)?;
    Ok(kept.len())
}

fn read_bounded_line(reader: &mut impl BufRead) -> Result<Option<Vec<u8>>, String> {
    let mut line = Vec::with_capacity(256);
    loop {
        let available = reader
            .fill_buf()
            .map_err(|_| "journal d'audit indisponible".to_string())?;
        if available.is_empty() {
            return Ok((!line.is_empty()).then_some(line));
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |position| position + 1);
        if line.len().saturating_add(take) > MAX_LINE_BYTES + 1 {
            return Err("journal d'audit invalide".to_string());
        }
        let content = if newline.is_some() { take - 1 } else { take };
        line.extend_from_slice(&available[..content]);
        reader.consume(take);
        if newline.is_some() {
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return Ok(Some(line));
        }
    }
}
