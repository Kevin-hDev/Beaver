const RETAINED_BYTES: usize = 1024 * 1024;
const PENDING_BYTES: usize = 28 * 1024;
const LIVE_PREVIEW_BYTES: usize = 50 * 1024;

#[derive(Clone, Copy)]
pub enum ShellStream {
    Stdout,
    Stderr,
}

pub struct PendingOutput {
    pub stdout: String,
    pub stderr: String,
    pub truncated: bool,
}

#[derive(Default)]
pub struct ShellOutputBuffer {
    retained: HeadTailBuffer,
    pending: PendingStreams,
}

impl ShellOutputBuffer {
    pub fn append(&mut self, stream: ShellStream, bytes: &[u8]) {
        self.retained.append(bytes, RETAINED_BYTES);
        self.pending.append(stream, bytes);
    }

    pub fn take_pending(&mut self) -> PendingOutput {
        std::mem::take(&mut self.pending).render()
    }

    #[cfg(test)]
    pub fn rendered(&self) -> String {
        render_string(&self.retained, RETAINED_BYTES)
    }

    pub fn live_preview(&self) -> String {
        let mut bytes = self.retained.tail_preview(LIVE_PREVIEW_BYTES);
        let output = String::from_utf8_lossy(&bytes).into_owned();
        zeroize::Zeroize::zeroize(&mut bytes);
        output
    }

    pub fn total_bytes(&self) -> usize {
        self.retained.total
    }
}

#[derive(Default)]
struct PendingStreams {
    stdout: HeadTailBuffer,
    stderr: HeadTailBuffer,
    total: usize,
}

impl PendingStreams {
    fn append(&mut self, stream: ShellStream, bytes: &[u8]) {
        self.total = self.total.saturating_add(bytes.len());
        match stream {
            ShellStream::Stdout => self.stdout.append(bytes, PENDING_BYTES),
            ShellStream::Stderr => self.stderr.append(bytes, PENDING_BYTES),
        }
    }

    fn render(self) -> PendingOutput {
        let (stdout_limit, stderr_limit) = stream_limits(self.stdout.total, self.stderr.total);
        PendingOutput {
            stdout: render_string(&self.stdout, stdout_limit),
            stderr: render_string(&self.stderr, stderr_limit),
            truncated: self.total > PENDING_BYTES,
        }
    }
}

fn stream_limits(stdout: usize, stderr: usize) -> (usize, usize) {
    if stdout.saturating_add(stderr) <= PENDING_BYTES {
        return (stdout, stderr);
    }
    let half = PENDING_BYTES / 2;
    let mut stdout_limit = stdout.min(half);
    let mut stderr_limit = stderr.min(half);
    let remaining = PENDING_BYTES.saturating_sub(stdout_limit + stderr_limit);
    let stdout_extra = stdout.saturating_sub(stdout_limit).min(remaining);
    stdout_limit += stdout_extra;
    stderr_limit += stderr
        .saturating_sub(stderr_limit)
        .min(remaining - stdout_extra);
    (stdout_limit, stderr_limit)
}

fn render_string(buffer: &HeadTailBuffer, limit: usize) -> String {
    let mut bytes = buffer.render(limit);
    let output = String::from_utf8_lossy(&bytes).into_owned();
    zeroize::Zeroize::zeroize(&mut bytes);
    output
}

impl Drop for HeadTailBuffer {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.head.zeroize();
        self.tail.iter_mut().for_each(|byte| *byte = 0);
    }
}

#[derive(Default)]
struct HeadTailBuffer {
    head: Vec<u8>,
    tail: std::collections::VecDeque<u8>,
    total: usize,
}

impl HeadTailBuffer {
    fn append(&mut self, bytes: &[u8], limit: usize) {
        self.total = self.total.saturating_add(bytes.len());
        let head_limit = limit / 2;
        let head_room = head_limit.saturating_sub(self.head.len());
        let head_bytes = head_room.min(bytes.len());
        self.head.extend_from_slice(&bytes[..head_bytes]);
        self.tail.extend(&bytes[head_bytes..]);

        let tail_limit = limit - head_limit;
        while self.tail.len() > tail_limit {
            if let Some(byte) = self.tail.front_mut() {
                *byte = 0;
            }
            self.tail.pop_front();
        }
    }

    fn render(&self, limit: usize) -> Vec<u8> {
        if self.total <= limit {
            let mut bytes = Vec::with_capacity(self.head.len() + self.tail.len());
            bytes.extend_from_slice(&self.head);
            bytes.extend(self.tail.iter());
            return bytes;
        }
        let head_count = self.head.len().min(limit / 2);
        let tail_count = self.tail.len().min(limit.saturating_sub(head_count));
        let mut bytes = Vec::with_capacity(head_count + tail_count + 64);
        bytes.extend_from_slice(&self.head[..head_count]);
        if self.total > head_count + tail_count {
            bytes.extend_from_slice(
                format!(
                    "\n... [{} octets omis] ...\n",
                    self.total - head_count - tail_count
                )
                .as_bytes(),
            );
        }
        bytes.extend(self.tail.iter().skip(self.tail.len() - tail_count));
        bytes
    }

    fn tail_preview(&self, limit: usize) -> Vec<u8> {
        let tail_count = self.tail.len().min(limit);
        let head_count = self.head.len().min(limit - tail_count);
        let mut bytes = Vec::with_capacity(head_count + tail_count);
        bytes.extend_from_slice(&self.head[self.head.len() - head_count..]);
        bytes.extend(self.tail.iter().skip(self.tail.len() - tail_count));
        bytes
    }
}

#[cfg(test)]
#[path = "tool_bash_output_tests.rs"]
mod tests;
