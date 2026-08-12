use crate::services::work_registry::ServiceWorkCancellation;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;

pub(super) const MAX_PROBE_STDOUT_BYTES: usize = 8 * 1024;
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

pub(super) struct ProbeSpec {
    program: PathBuf,
    args: Vec<OsString>,
}

impl ProbeSpec {
    pub(super) fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub(super) fn args(mut self, args: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
}

pub(super) struct ProbeOutput {
    pub(super) stdout: Vec<u8>,
    pub(super) truncated: bool,
}

pub(super) async fn run(spec: ProbeSpec, cancel: &ServiceWorkCancellation) -> Option<ProbeOutput> {
    let output = run_with_observer(spec, cancel, |_| {}).await?;
    if output.truncated {
        ::log::warn!("[gpu-probe] stdout truncated");
    }
    Some(output)
}

async fn run_with_observer<Observer>(
    spec: ProbeSpec,
    cancel: &ServiceWorkCancellation,
    observer: Observer,
) -> Option<ProbeOutput>
where
    Observer: FnOnce(u32) + Send,
{
    if cancel.is_cancelled() {
        return None;
    }
    let mut command = crate::services::background_command::new_tokio(&spec.program);
    command
        .args(spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = crate::services::owned_process::OwnedProcess::spawn_tokio(
        &mut command,
        crate::services::process_tree::ProcessKind::GpuProbe,
    )
    .await
    .ok()?;
    let pid = child.id()?;
    observer(pid);
    let stdout = child.stdout.take()?;
    let reader = tokio::spawn(read_bounded_and_drain(stdout));
    enum ProbeEnd {
        Exited(bool),
        Cancelled,
    }
    let end = tokio::select! {
        result = child.wait() => ProbeEnd::Exited(result.is_ok_and(|status| status.success())),
        _ = cancel.cancelled() => ProbeEnd::Cancelled,
        _ = tokio::time::sleep(PROBE_TIMEOUT) => ProbeEnd::Cancelled,
    };
    let succeeded = match end {
        ProbeEnd::Exited(succeeded) => {
            crate::services::owned_process::release(pid);
            succeeded
        }
        ProbeEnd::Cancelled => {
            crate::services::process_tree::terminate_tokio(
                &mut child,
                crate::services::process_tree::ProcessKind::GpuProbe,
            )
            .await;
            false
        }
    };
    let output = reader.await.ok()??;
    if !succeeded {
        return None;
    }
    Some(output)
}

async fn read_bounded_and_drain(stdout: tokio::process::ChildStdout) -> Option<ProbeOutput> {
    let mut stdout = stdout;
    let mut collected = Vec::with_capacity(MAX_PROBE_STDOUT_BYTES);
    let mut chunk = [0_u8; 1024];
    let mut truncated = false;
    loop {
        let count = stdout.read(&mut chunk).await.ok()?;
        if count == 0 {
            break;
        }
        let remaining = MAX_PROBE_STDOUT_BYTES.saturating_sub(collected.len());
        collected.extend_from_slice(&chunk[..count.min(remaining)]);
        truncated |= count > remaining;
    }
    Some(ProbeOutput {
        stdout: collected,
        truncated,
    })
}

#[cfg(test)]
pub(super) async fn run_for_test(
    spec: ProbeSpec,
    cancel: &ServiceWorkCancellation,
    started: tokio::sync::oneshot::Sender<u32>,
) -> Option<ProbeOutput> {
    run_with_observer(spec, cancel, move |pid| {
        let _ = started.send(pid);
    })
    .await
}
