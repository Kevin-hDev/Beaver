use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;

const MAX_PROBE_BYTES: usize = 64;

pub(super) async fn run<Observer>(
    command: &mut tokio::process::Command,
    timeout: Duration,
    observer: Observer,
) -> Option<Vec<u8>>
where
    Observer: FnOnce(u32) + Send,
{
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = crate::services::owned_process::OwnedProcess::spawn_tokio(
        command,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await
    .ok()?;
    let pid = child.id()?;
    observer(pid);
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate(&mut child).await;
            return None;
        }
    };
    let deadline = tokio::time::Instant::now() + timeout;
    let mut reader = tokio::spawn(read_bounded(stdout));
    let status = tokio::select! {
        result = child.wait() => Some(result.ok()?),
        _ = tokio::time::sleep_until(deadline) => None,
    };
    let Some(status) = status else {
        terminate(&mut child).await;
        reader.abort();
        let _ = reader.await;
        return None;
    };
    crate::services::owned_process::release(pid);
    if !status.success() {
        reader.abort();
        let _ = reader.await;
        return None;
    }
    tokio::time::timeout_at(deadline, &mut reader)
        .await
        .ok()?
        .ok()?
}

async fn terminate(child: &mut tokio::process::Child) {
    crate::services::process_tree::terminate_tokio(
        child,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await;
}

async fn read_bounded(mut stdout: tokio::process::ChildStdout) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(MAX_PROBE_BYTES);
    let mut chunk = [0_u8; 32];
    let mut oversized = false;
    loop {
        let read = stdout.read(&mut chunk).await.ok()?;
        if read == 0 {
            break;
        }
        let remaining = MAX_PROBE_BYTES.saturating_sub(output.len());
        output.extend_from_slice(&chunk[..read.min(remaining)]);
        oversized |= read > remaining;
    }
    (!oversized).then_some(output)
}
