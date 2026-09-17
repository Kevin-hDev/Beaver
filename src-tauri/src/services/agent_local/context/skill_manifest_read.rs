use std::io::Read;
use std::path::Path;

use super::skill_limits::MAX_SKILL_CONTENT_BYTES;

pub(super) fn read(path: &Path) -> Result<String, ()> {
    read_with(path, || {})
}

#[cfg(test)]
pub(super) fn read_after_metadata<F>(path: &Path, after_metadata: F) -> Result<String, ()>
where
    F: FnOnce(),
{
    read_with(path, after_metadata)
}

fn read_with<F>(path: &Path, after_metadata: F) -> Result<String, ()>
where
    F: FnOnce(),
{
    let mut file = crate::services::private_store::open_regular_single_link(path)
        .map_err(|_| ())?
        .ok_or(())?;
    let metadata = file.metadata().map_err(|_| ())?;
    if !metadata.is_file() || metadata.len() > MAX_SKILL_CONTENT_BYTES as u64 {
        return Err(());
    }
    after_metadata();
    let read_limit = (MAX_SKILL_CONTENT_BYTES as u64).checked_add(1).ok_or(())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(&mut file)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() > MAX_SKILL_CONTENT_BYTES {
        return Err(());
    }
    String::from_utf8(bytes).map_err(|_| ())
}
