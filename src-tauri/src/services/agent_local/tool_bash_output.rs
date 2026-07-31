const RETAINED_BYTES: usize = 1024 * 1024;
const PENDING_BYTES: usize = 28 * 1024;
const LIVE_PREVIEW_BYTES: usize = 50 * 1024;

#[derive(Default)]
pub struct ShellOutputBuffer {
    retained: HeadTailBuffer,
    pending: HeadTailBuffer,
}

impl ShellOutputBuffer {
    pub fn append(&mut self, bytes: &[u8]) {
        self.retained.append(bytes, RETAINED_BYTES);
        self.pending.append(bytes, PENDING_BYTES);
    }

    pub fn take_pending(&mut self) -> String {
        let mut bytes = std::mem::take(&mut self.pending).render(PENDING_BYTES);
        let output = String::from_utf8_lossy(&bytes).into_owned();
        zeroize::Zeroize::zeroize(&mut bytes);
        output
    }

    #[cfg(test)]
    pub fn rendered(&self) -> String {
        let mut bytes = self.retained.render(RETAINED_BYTES);
        let output = String::from_utf8_lossy(&bytes).into_owned();
        zeroize::Zeroize::zeroize(&mut bytes);
        output
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

    pub fn is_truncated(&self) -> bool {
        self.retained.total > PENDING_BYTES
    }
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
        if self.tail.len() > tail_limit {
            let overflow = self.tail.len() - tail_limit;
            for _ in 0..overflow {
                if let Some(byte) = self.tail.front_mut() {
                    *byte = 0;
                }
                self.tail.pop_front();
            }
        }
    }

    fn render(&self, limit: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.head.len() + self.tail.len() + 64);
        bytes.extend_from_slice(&self.head);
        if self.total > limit {
            bytes.extend_from_slice(
                format!("\n... [{} octets omis] ...\n", self.total - limit).as_bytes(),
            );
        }
        bytes.extend(self.tail.iter());
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
