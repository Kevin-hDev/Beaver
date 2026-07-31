use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::mpsc;

use super::tool_bash_output::ShellStream;
use super::tool_bash_session::ShellSession;
use super::tool_bash_storage::ShellOutputStore;

pub const OUTPUT_CHANNEL_SIZE: usize = 64;
const READ_CHUNK_SIZE: usize = 8 * 1024;
const DRAIN_MAX_TIMEOUT_SECS: u64 = 2;

pub enum OutputEvent {
    Data(ShellStream, Vec<u8>),
    Failed,
}

pub enum DrainOutcome {
    Complete,
    TimedOut,
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
    stream: ShellStream,
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
                        .send(OutputEvent::Data(stream, buffer[..count].to_vec()))
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
) -> DrainOutcome {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(DRAIN_MAX_TIMEOUT_SECS);
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return DrainOutcome::TimedOut;
        }
        match tokio::time::timeout(remaining, receiver.recv()).await {
            Ok(Some(OutputEvent::Data(stream, mut bytes))) => {
                use zeroize::Zeroize;
                if store.append(&bytes).await.is_err() {
                    bytes.zeroize();
                    return DrainOutcome::Failed;
                }
                session.append_output(stream, &bytes);
                bytes.zeroize();
            }
            Ok(Some(OutputEvent::Failed)) => return DrainOutcome::Failed,
            Err(_) => return DrainOutcome::TimedOut,
            Ok(None) => return DrainOutcome::Complete,
        }
    }
}

pub fn clear_pending(receiver: &mut mpsc::Receiver<OutputEvent>) {
    use zeroize::Zeroize;
    while let Ok(event) = receiver.try_recv() {
        if let OutputEvent::Data(_, mut bytes) = event {
            bytes.zeroize();
        }
    }
}
