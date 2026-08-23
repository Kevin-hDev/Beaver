use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use zeroize::{Zeroize, Zeroizing};

use super::ScanError;

pub(super) struct DecodedString {
    bytes: Zeroizing<Vec<u8>>,
    max_bytes: usize,
    zeroized: Option<Arc<AtomicBool>>,
}

impl DecodedString {
    fn new(max_bytes: usize, zeroized: Option<Arc<AtomicBool>>) -> Self {
        if let Some(observer) = &zeroized {
            observer.store(false, Ordering::SeqCst);
        }
        Self {
            bytes: Zeroizing::new(Vec::with_capacity(max_bytes)),
            max_bytes,
            zeroized,
        }
    }

    fn push(&mut self, value: &[u8]) -> Result<(), ScanError> {
        let next_len = self.bytes.len().checked_add(value.len()).ok_or(ScanError)?;
        if next_len > self.max_bytes {
            return Err(ScanError);
        }
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    pub(super) fn as_str(&self) -> Result<&str, ScanError> {
        std::str::from_utf8(&self.bytes).map_err(|_| ScanError)
    }

    pub(super) fn erase(&mut self) {
        self.bytes.as_mut_slice().zeroize();
        if let Some(observer) = &self.zeroized {
            observer.store(self.bytes.iter().all(|byte| *byte == 0), Ordering::SeqCst);
        }
        self.bytes.clear();
    }
}

impl Drop for DecodedString {
    fn drop(&mut self) {
        self.erase();
    }
}

pub(super) fn decode(
    body: &[u8],
    cursor: &mut usize,
    max_bytes: usize,
) -> Result<DecodedString, ScanError> {
    decode_with_zeroize_hook(body, cursor, max_bytes, None)
}

pub(super) fn decode_with_zeroize_hook(
    body: &[u8],
    cursor: &mut usize,
    max_bytes: usize,
    zeroized: Option<Arc<AtomicBool>>,
) -> Result<DecodedString, ScanError> {
    let mut decoded = DecodedString::new(max_bytes, zeroized);
    scan(body, cursor, Some(&mut decoded))?;
    decoded.as_str()?;
    Ok(decoded)
}

pub(super) fn skip(body: &[u8], cursor: &mut usize) -> Result<(), ScanError> {
    // Les valeurs ignorées sont validées directement dans l'entrée, sans buffer décodé.
    scan(body, cursor, None)
}

fn scan(
    body: &[u8],
    cursor: &mut usize,
    mut decoded: Option<&mut DecodedString>,
) -> Result<(), ScanError> {
    if body.get(*cursor) != Some(&b'"') {
        return Err(ScanError);
    }
    *cursor += 1;
    loop {
        match body.get(*cursor).copied() {
            Some(b'"') => {
                *cursor += 1;
                return Ok(());
            }
            Some(b'\\') => {
                *cursor += 1;
                escape(body, cursor, &mut decoded)?;
            }
            Some(0x00..=0x1f) | None => return Err(ScanError),
            Some(byte) => {
                *cursor += 1;
                emit(&mut decoded, &[byte])?;
            }
        }
    }
}

fn escape(
    body: &[u8],
    cursor: &mut usize,
    decoded: &mut Option<&mut DecodedString>,
) -> Result<(), ScanError> {
    let escaped = body.get(*cursor).copied().ok_or(ScanError)?;
    *cursor += 1;
    match escaped {
        b'"' | b'\\' | b'/' => emit(decoded, &[escaped]),
        b'b' => emit(decoded, &[0x08]),
        b'f' => emit(decoded, &[0x0c]),
        b'n' => emit(decoded, b"\n"),
        b'r' => emit(decoded, b"\r"),
        b't' => emit(decoded, b"\t"),
        b'u' => unicode(body, cursor, decoded),
        _ => Err(ScanError),
    }
}

fn unicode(
    body: &[u8],
    cursor: &mut usize,
    decoded: &mut Option<&mut DecodedString>,
) -> Result<(), ScanError> {
    let first = hex_unit(body, cursor)?;
    let second = match first {
        0xd800..=0xdbff => {
            if body.get(*cursor..cursor.saturating_add(2)) != Some(b"\\u") {
                return Err(ScanError);
            }
            *cursor += 2;
            let second = hex_unit(body, cursor)?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(ScanError);
            }
            Some(second)
        }
        0xdc00..=0xdfff => return Err(ScanError),
        _ => None,
    };
    // Une valeur ignorée ne produit ni scalaire Unicode ni octets décodés.
    if decoded.is_none() {
        return Ok(());
    }
    let mut scalar = second.map_or_else(
        || u32::from(first),
        |low| 0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(low) - 0xdc00),
    );
    let character = char::from_u32(scalar).ok_or(ScanError)?;
    let mut bytes = [0_u8; 4];
    let result = emit(decoded, character.encode_utf8(&mut bytes).as_bytes());
    bytes.zeroize();
    scalar.zeroize();
    result
}

fn hex_unit(body: &[u8], cursor: &mut usize) -> Result<u16, ScanError> {
    let end = cursor.checked_add(4).ok_or(ScanError)?;
    let digits = body.get(*cursor..end).ok_or(ScanError)?;
    let mut value = 0_u16;
    for digit in digits {
        let nibble = u16::from(hex(*digit)?);
        value = value
            .checked_mul(16)
            .and_then(|current| current.checked_add(nibble))
            .ok_or(ScanError)?;
    }
    *cursor = end;
    Ok(value)
}

fn hex(byte: u8) -> Result<u8, ScanError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(ScanError),
    }
}

fn emit(decoded: &mut Option<&mut DecodedString>, value: &[u8]) -> Result<(), ScanError> {
    if let Some(output) = decoded.as_deref_mut() {
        output.push(value)?;
    }
    Ok(())
}
