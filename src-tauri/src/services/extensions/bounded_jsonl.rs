use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

pub(super) const MAX_LOG_BYTES: usize = 64 * 1024;
static WRITES: Mutex<()> = Mutex::new(());

pub(super) fn write(path: &Path, entry: &impl Serialize) -> Result<(), String> {
    let _guard = WRITES.lock().map_err(|_| unavailable())?;
    let mut line = serde_json::to_vec(entry).map_err(|_| unavailable())?;
    line.push(b'\n');
    if line.len() > MAX_LOG_BYTES {
        return Err(unavailable());
    }
    let mut contents = existing(path)?;
    while contents.len().saturating_add(line.len()) > MAX_LOG_BYTES {
        let Some(position) = contents.iter().position(|byte| *byte == b'\n') else {
            contents.clear();
            break;
        };
        contents.drain(..=position);
    }
    contents.extend_from_slice(&line);
    crate::services::private_store::atomic_write(path, &contents).map_err(|_| unavailable())
}

fn existing(path: &Path) -> Result<Vec<u8>, String> {
    match crate::services::private_store::read_bounded_regular(path, MAX_LOG_BYTES as u64) {
        Ok(crate::services::private_store::BoundedFile::Missing) => Ok(Vec::new()),
        Ok(crate::services::private_store::BoundedFile::Content(bytes)) => Ok(bytes),
        Err(_) => {
            ::log::error!("[extensions] invalid extension journal reset");
            Ok(Vec::new())
        }
    }
}

fn unavailable() -> String {
    "journal d'extensions indisponible".to_string()
}
