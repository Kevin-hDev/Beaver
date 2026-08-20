use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::task::JoinSet;
use tokio::time::Instant;

use crate::services::work_registry::ServiceWorkCancellation;

const STREAM_TAIL_BYTES: usize = 4_096;
const ABORT_DRAIN_TIMEOUT: Duration = Duration::from_millis(100);

pub(super) async fn drain<R>(mut reader: R) -> DrainedTail
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut tail = OutputTail::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match reader.read(&mut chunk).await {
            Ok(0) => {
                return DrainedTail {
                    tail,
                    failed: false,
                }
            }
            Ok(read) => tail.push(&chunk[..read]),
            Err(_) => return DrainedTail { tail, failed: true },
        }
    }
}

pub(super) struct DrainedTail {
    pub(super) tail: OutputTail,
    pub(super) failed: bool,
}

pub(super) struct DrainedOutput {
    pub(super) stdout: OutputTail,
    pub(super) stderr: OutputTail,
    pub(super) failed: bool,
}

pub(super) enum DrainWait {
    Ready,
    Failed,
    Cancelled,
    TimedOut,
}

pub(super) struct DrainTasks {
    tasks: JoinSet<(Stream, DrainedTail)>,
    stdout: OutputTail,
    stderr: OutputTail,
    failed: bool,
}

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

impl DrainTasks {
    pub(super) fn start(
        stdout: tokio::process::ChildStdout,
        stderr: tokio::process::ChildStderr,
    ) -> Self {
        let mut tasks = JoinSet::new();
        tasks.spawn(async move { (Stream::Stdout, drain(stdout).await) });
        tasks.spawn(async move { (Stream::Stderr, drain(stderr).await) });
        Self {
            tasks,
            stdout: OutputTail::new(),
            stderr: OutputTail::new(),
            failed: false,
        }
    }

    pub(super) async fn wait(
        &mut self,
        deadline: Instant,
        cancel: &ServiceWorkCancellation,
    ) -> DrainWait {
        loop {
            if self.tasks.is_empty() {
                return if self.failed {
                    DrainWait::Failed
                } else {
                    DrainWait::Ready
                };
            }
            if Instant::now() >= deadline {
                return DrainWait::TimedOut;
            }
            tokio::select! {
                result = self.tasks.join_next() => {
                    if !self.store(result) {
                        return DrainWait::Failed;
                    }
                }
                _ = cancel.cancelled() => return DrainWait::Cancelled,
                _ = tokio::time::sleep_until(deadline) => return DrainWait::TimedOut,
            }
        }
    }

    pub(super) async fn abort_and_collect(&mut self) -> DrainedOutput {
        self.tasks.abort_all();
        let deadline = Instant::now() + ABORT_DRAIN_TIMEOUT;
        while !self.tasks.is_empty() && Instant::now() < deadline {
            tokio::select! {
                result = self.tasks.join_next() => {
                    let _ = self.store(result);
                }
                _ = tokio::time::sleep_until(deadline) => break,
            }
        }
        self.take_output()
    }

    fn store(
        &mut self,
        result: Option<Result<(Stream, DrainedTail), tokio::task::JoinError>>,
    ) -> bool {
        match result {
            Some(Ok((Stream::Stdout, drained))) => {
                self.stdout = drained.tail;
                self.failed |= drained.failed;
                true
            }
            Some(Ok((Stream::Stderr, drained))) => {
                self.stderr = drained.tail;
                self.failed |= drained.failed;
                true
            }
            Some(Err(_)) | None => {
                self.failed = true;
                false
            }
        }
    }

    pub(super) fn take_output(&mut self) -> DrainedOutput {
        DrainedOutput {
            stdout: std::mem::replace(&mut self.stdout, OutputTail::new()),
            stderr: std::mem::replace(&mut self.stderr, OutputTail::new()),
            failed: self.failed,
        }
    }
}

pub(super) struct OutputTail {
    bytes: [u8; STREAM_TAIL_BYTES],
    start: usize,
    len: usize,
}

impl OutputTail {
    pub(super) fn new() -> Self {
        Self {
            bytes: [0; STREAM_TAIL_BYTES],
            start: 0,
            len: 0,
        }
    }

    fn push(&mut self, data: &[u8]) {
        for byte in data {
            let index = (self.start + self.len) % STREAM_TAIL_BYTES;
            self.bytes[index] = *byte;
            if self.len < STREAM_TAIL_BYTES {
                self.len += 1;
            } else {
                self.start = (self.start + 1) % STREAM_TAIL_BYTES;
            }
        }
    }

    pub(super) fn as_bytes(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(self.len);
        for offset in 0..self.len {
            output.push(self.bytes[(self.start + offset) % STREAM_TAIL_BYTES]);
        }
        output
    }
}
