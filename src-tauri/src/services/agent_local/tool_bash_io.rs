use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::mpsc;

use super::tool_bash_session::ShellSession;
use super::tool_bash_storage::ShellOutputStore;

pub const OUTPUT_CHANNEL_SIZE: usize = 64;
const READ_CHUNK_SIZE: usize = 8 * 1024;
const DRAIN_IDLE_TIMEOUT_MS: u64 = 100;
const DRAIN_MAX_TIMEOUT_SECS: u64 = 2;

pub enum OutputEvent {
    Data(Vec<u8>),
    Failed,
}

pub async fn read_bounded<R: AsyncRead + Unpin>(
    mut reader: R,
    limit: usize,
) -> std::io::Result<(Vec<u8>, bool)> {
    use zeroize::Zeroize;

    let mut output = Vec::with_capacity(limit.min(32 * 1024));
    let mut chunk = [0_u8; READ_CHUNK_SIZE];
    let mut exceeded = false;
    loop {
        let count = match reader.read(&mut chunk).await {
            Ok(count) => count,
            Err(error) => {
                output.zeroize();
                chunk.zeroize();
                return Err(error);
            }
        };
        if count == 0 {
            break;
        }
        let room = limit.saturating_sub(output.len());
        output.extend_from_slice(&chunk[..count.min(room)]);
        exceeded |= count > room;
    }
    chunk.zeroize();
    Ok((output, exceeded))
}

pub fn spawn_reader<R>(
    mut reader: R,
    sender: mpsc::Sender<OutputEvent>,
) -> tokio::task::JoinHandle<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut buffer = zeroize::Zeroizing::new(vec![0_u8; READ_CHUNK_SIZE]);
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(count) => {
                    if sender
                        .send(OutputEvent::Data(buffer[..count].to_vec()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(_) => {
                    let _ = sender.send(OutputEvent::Failed).await;
                    break;
                }
            }
        }
    })
}

pub async fn drain(
    session: &ShellSession,
    store: &mut ShellOutputStore,
    receiver: &mut mpsc::Receiver<OutputEvent>,
) -> bool {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(DRAIN_MAX_TIMEOUT_SECS);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return false;
        }
        let wait = remaining.min(Duration::from_millis(DRAIN_IDLE_TIMEOUT_MS));
        match tokio::time::timeout(wait, receiver.recv()).await {
            Ok(Some(OutputEvent::Data(mut bytes))) => {
                use zeroize::Zeroize;
                if store.append(&bytes).await.is_err() {
                    bytes.zeroize();
                    return false;
                }
                session.append_output(&bytes);
                bytes.zeroize();
            }
            Ok(Some(OutputEvent::Failed)) | Err(_) => return false,
            Ok(None) => return true,
        }
    }
}

pub fn clear_pending(receiver: &mut mpsc::Receiver<OutputEvent>) {
    use zeroize::Zeroize;
    while let Ok(event) = receiver.try_recv() {
        if let OutputEvent::Data(mut bytes) = event {
            bytes.zeroize();
        }
    }
}
