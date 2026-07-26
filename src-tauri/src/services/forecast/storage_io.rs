use std::path::Path;
use tokio::io::AsyncReadExt;

pub async fn read_bounded(path: &Path, max_bytes: usize) -> std::io::Result<Vec<u8>> {
    let file = tokio::fs::File::open(path).await?;
    let mut data = Vec::with_capacity(max_bytes.min(64 * 1024));
    file.take((max_bytes + 1) as u64)
        .read_to_end(&mut data)
        .await?;
    if data.len() > max_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "forecast file too large",
        ));
    }
    Ok(data)
}
