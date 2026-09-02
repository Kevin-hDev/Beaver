use std::io::Read;
use std::path::Path;

pub fn read_bounded(path: &Path, limit: usize, overflow: &str) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|_| "cannot read contract input".to_string())?;
    let mut bytes = Vec::with_capacity(limit.min(8_192).saturating_add(1));
    file.take(limit.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read contract input".to_string())?;
    if bytes.len() > limit {
        return Err(overflow.to_string());
    }
    Ok(bytes)
}
