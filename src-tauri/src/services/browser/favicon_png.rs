use super::favicon_policy::{valid_dimensions, MAX_PNG_BYTES};
use base64::{engine::general_purpose::STANDARD, Engine};

pub(super) fn encode(
    width: i32,
    height: i32,
    size: usize,
    read: impl FnOnce(&mut Vec<u8>) -> usize,
) -> Option<String> {
    if !valid_dimensions(width, height) || size == 0 || size > MAX_PNG_BYTES {
        return None;
    }
    let mut bytes = vec![0; size];
    if read(&mut bytes) != size || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }
    Some(STANDARD.encode(bytes))
}
