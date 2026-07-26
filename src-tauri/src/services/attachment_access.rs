use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use zeroize::Zeroize;

pub const MAX_ATTACHMENTS: usize = 15;
pub const MAX_ATTACHMENT_SIZE: u64 = 20 * 1024 * 1024;
const MAX_PATH_BYTES: usize = 4096;
const GRANT_PREFIX: &str = "v1.";
const GRANT_DOMAIN: &[u8] = b"cl-go-dash:attachment-access:v1";
const ERROR_CODE: &str = "attachment_access_denied";

#[derive(Debug, Serialize)]
pub struct RegisteredAttachment {
    pub path: String,
    pub size: u64,
    pub access_grant: String,
}

pub fn register_paths<F>(
    paths: &[String],
    key: &[u8],
    is_allowed: F,
) -> Result<Vec<RegisteredAttachment>, String>
where
    F: Fn(&Path) -> bool,
{
    if paths.is_empty() || paths.len() > MAX_ATTACHMENTS || key.len() != 32 {
        return Err(ERROR_CODE.into());
    }
    let mut unique = HashSet::with_capacity(paths.len());
    let mut registered = Vec::with_capacity(paths.len());
    for raw in paths {
        let raw_path = validate_raw_path(raw)?;
        let (canonical, size) = validate_file(raw_path, MAX_ATTACHMENT_SIZE)?;
        if !is_allowed(raw_path) && !is_allowed(&canonical) {
            return Err(ERROR_CODE.into());
        }
        let canonical_text = canonical.to_str().ok_or(ERROR_CODE)?.to_string();
        if !unique.insert(canonical_text.clone()) {
            return Err(ERROR_CODE.into());
        }
        registered.push(RegisteredAttachment {
            access_grant: create_grant(&canonical_text, key)?,
            path: raw.clone(),
            size,
        });
    }
    Ok(registered)
}

pub fn verify_access_grant(raw: &str, access_grant: &str, key: &[u8]) -> Result<PathBuf, String> {
    if key.len() != 32 {
        return Err(ERROR_CODE.into());
    }
    let raw_path = validate_raw_path(raw)?;
    let (canonical, _) = validate_file(raw_path, MAX_ATTACHMENT_SIZE)?;
    let canonical_text = canonical.to_str().ok_or(ERROR_CODE)?;
    let encoded = access_grant.strip_prefix(GRANT_PREFIX).ok_or(ERROR_CODE)?;
    if encoded.len() != 64 {
        return Err(ERROR_CODE.into());
    }
    let mut received = [0_u8; 32];
    hex::decode_to_slice(encoded, &mut received).map_err(|_| ERROR_CODE.to_string())?;
    let mut expected = compute_mac(canonical_text, key)?;
    let valid = constant_time_32(&received, &expected);
    received.zeroize();
    expected.zeroize();
    if !valid {
        return Err(ERROR_CODE.into());
    }
    Ok(canonical)
}

pub fn selected_file<F>(raw: &str, max_size: u64, is_allowed: F) -> Result<PathBuf, String>
where
    F: Fn(&Path) -> bool,
{
    let raw_path = validate_raw_path(raw)?;
    let (canonical, _) = validate_file(raw_path, max_size)?;
    if !is_allowed(raw_path) && !is_allowed(&canonical) {
        return Err(ERROR_CODE.into());
    }
    Ok(canonical)
}

fn create_grant(canonical: &str, key: &[u8]) -> Result<String, String> {
    let mut digest = compute_mac(canonical, key)?;
    let grant = format!("{GRANT_PREFIX}{}", hex::encode(digest));
    digest.zeroize();
    Ok(grant)
}

fn compute_mac(canonical: &str, key: &[u8]) -> Result<[u8; 32], String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| ERROR_CODE.to_string())?;
    mac.update(GRANT_DOMAIN);
    mac.update(&(canonical.len() as u64).to_be_bytes());
    mac.update(canonical.as_bytes());
    Ok(mac.finalize().into_bytes().into())
}

fn constant_time_32(left: &[u8; 32], right: &[u8; 32]) -> bool {
    let mut different = 0_u8;
    for index in 0..32 {
        different |= left[index] ^ right[index];
    }
    different == 0
}

fn validate_raw_path(raw: &str) -> Result<&Path, String> {
    if raw.is_empty() || raw.len() > MAX_PATH_BYTES || raw.chars().any(char::is_control) {
        return Err(ERROR_CODE.into());
    }
    let path = Path::new(raw);
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ERROR_CODE.into());
    }
    Ok(path)
}

fn validate_file(path: &Path, max_size: u64) -> Result<(PathBuf, u64), String> {
    let source = path
        .symlink_metadata()
        .map_err(|_| ERROR_CODE.to_string())?;
    if source.file_type().is_symlink() || !source.is_file() {
        return Err(ERROR_CODE.into());
    }
    let canonical = path.canonicalize().map_err(|_| ERROR_CODE.to_string())?;
    let metadata = canonical.metadata().map_err(|_| ERROR_CODE.to_string())?;
    if !metadata.is_file() || metadata.len() > max_size {
        return Err(ERROR_CODE.into());
    }
    Ok((canonical, metadata.len()))
}
