use tokio::io::{AsyncBufRead, AsyncBufReadExt};

pub async fn read_bounded_line<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    limit: usize,
) -> Result<Option<Vec<u8>>, String> {
    let mut line = Vec::with_capacity(limit.min(8192));
    loop {
        let available = reader
            .fill_buf()
            .await
            .map_err(|_| "lecture MCP impossible".to_string())?;
        if available.is_empty() {
            return Ok((!line.is_empty()).then_some(line));
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(take) > limit {
            return Err("réponse MCP trop volumineuse".to_string());
        }
        let complete = available.get(take.saturating_sub(1)) == Some(&b'\n');
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if complete {
            return Ok(Some(line));
        }
    }
}
