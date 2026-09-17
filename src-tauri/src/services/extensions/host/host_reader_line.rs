use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;

pub(super) async fn read_bounded_line(
    reader: &mut BufReader<ChildStdout>,
) -> Result<Vec<u8>, String> {
    let mut line = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .await
            .map_err(|_| "Hôte d'extensions indisponible.".to_string())?;
        if available.is_empty() {
            return Err("L'hôte d'extensions s'est arrêté.".to_string());
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if line.len().saturating_add(take) > super::types::MAX_MESSAGE_BYTES {
            return Err("Réponse de l'hôte d'extensions trop volumineuse.".to_string());
        }
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if line.last() == Some(&b'\n') {
            line.pop();
            return Ok(line);
        }
    }
}
