#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::extract::validate_member_path;
use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const EOCD_BYTES: usize = 22;
const MAX_EOCD_SEARCH: u64 = 65_557;
const MAX_MEMBER_NAME_BYTES: usize = 4 * 1024;

pub(super) fn validate_zip_directory(path: &Path) -> Result<(), OllamaErrorCode> {
    let mut file = std::fs::File::open(path).map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let length = file
        .metadata()
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?
        .len();
    let tail_length = length.min(MAX_EOCD_SEARCH);
    file.seek(SeekFrom::Start(length.saturating_sub(tail_length)))
        .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
    let mut tail = vec![0_u8; usize::try_from(tail_length).unwrap_or(0)];
    file.read_exact(&mut tail)
        .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
    let eocd = tail
        .windows(4)
        .rposition(|window| window == b"PK\x05\x06")
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
    if tail.len().saturating_sub(eocd) < EOCD_BYTES {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    let entries_on_disk = read_u16(&tail, eocd + 8)?;
    let entries = read_u16(&tail, eocd + 10)?;
    if entries_on_disk != entries || entries == u16::MAX {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    let central_size = u64::from(read_u32(&tail, eocd + 12)?);
    let central_offset = u64::from(read_u32(&tail, eocd + 16)?);
    let central_end = central_offset
        .checked_add(central_size)
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
    if central_end > length {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    file.seek(SeekFrom::Start(central_offset))
        .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
    let mut names = HashSet::with_capacity(usize::from(entries));
    for _ in 0..entries {
        let mut header = [0_u8; 46];
        file.read_exact(&mut header)
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        if &header[..4] != b"PK\x01\x02" {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let name_length = usize::from(read_u16(&header, 28)?);
        let extra_length = u64::from(read_u16(&header, 30)?);
        let comment_length = u64::from(read_u16(&header, 32)?);
        if name_length == 0 || name_length > MAX_MEMBER_NAME_BYTES {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let mut name = vec![0_u8; name_length];
        file.read_exact(&mut name)
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        let name = String::from_utf8(name).map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
        validate_member_path(Path::new(&name))?;
        if !names.insert(name) {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        let skip = extra_length
            .checked_add(comment_length)
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        file.seek(SeekFrom::Current(i64::try_from(skip).unwrap_or(i64::MAX)))
            .map_err(|_| OllamaErrorCode::OllamaBundleInvalid)?;
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, OllamaErrorCode> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, OllamaErrorCode> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}
