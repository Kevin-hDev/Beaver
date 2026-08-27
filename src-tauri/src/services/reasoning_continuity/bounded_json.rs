use std::io::{self, Write};

use serde::Serialize;

use super::limits::LimitError;

pub fn serialized_len_bounded<T: Serialize + ?Sized>(
    value: &T,
    maximum: usize,
) -> Result<usize, LimitError> {
    serialized_len_bounded_from(value, 0, maximum)
}

pub fn serialized_len_bounded_from<T: Serialize + ?Sized>(
    value: &T,
    initial: usize,
    maximum: usize,
) -> Result<usize, LimitError> {
    let mut writer = BoundedWriter::new(initial, maximum)?;
    serde_json::to_writer(&mut writer, value).map_err(|_| LimitError::EnvelopeBytes)?;
    Ok(writer.written())
}

struct BoundedWriter {
    written: usize,
    maximum: usize,
}

impl BoundedWriter {
    fn new(initial: usize, maximum: usize) -> Result<Self, LimitError> {
        (initial <= maximum)
            .then_some(Self {
                written: initial,
                maximum,
            })
            .ok_or(LimitError::EnvelopeBytes)
    }

    const fn written(&self) -> usize {
        self.written
    }
}

impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let remaining = self.maximum.saturating_sub(self.written);
        if remaining == 0 {
            return Err(io::Error::other("reasoning_continuity_limit"));
        }
        let accepted = remaining.min(bytes.len());
        self.written += accepted;
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
