pub(super) struct Utf8StreamDecoder {
    pending: Vec<u8>,
}

impl Utf8StreamDecoder {
    pub(super) fn new() -> Self {
        Self {
            pending: Vec::with_capacity(3),
        }
    }

    pub(super) fn push(&mut self, bytes: &[u8]) -> String {
        let mut input = Vec::with_capacity(self.pending.len() + bytes.len());
        input.append(&mut self.pending);
        input.extend_from_slice(bytes);
        let mut remaining = input.as_slice();
        let mut decoded = String::new();

        while !remaining.is_empty() {
            match std::str::from_utf8(remaining) {
                Ok(valid) => {
                    decoded.push_str(valid);
                    break;
                }
                Err(error) => {
                    let valid_len = error.valid_up_to();
                    if valid_len > 0 {
                        let valid = std::str::from_utf8(&remaining[..valid_len])
                            .expect("prefix validated by Utf8Error");
                        decoded.push_str(valid);
                    }
                    remaining = &remaining[valid_len..];
                    if let Some(invalid_len) = error.error_len() {
                        decoded.push('\u{fffd}');
                        remaining = &remaining[invalid_len..];
                    } else {
                        debug_assert!(remaining.len() <= 3);
                        self.pending.extend_from_slice(remaining);
                        break;
                    }
                }
            }
        }

        decoded
    }

    pub(super) fn finish(self) -> String {
        if self.pending.is_empty() {
            String::new()
        } else {
            "\u{fffd}".to_string()
        }
    }
}
