#![allow(dead_code)]

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

use super::constants::MAX_PROBE_RESPONSE_BYTES;
use super::error::OllamaErrorCode;
use super::fingerprint::OllamaVersion;
use super::fingerprint::Sha256Digest;
use super::probe::{PreparedBundle, TargetValidation};
use super::spawn_profile::OllamaSpawnProfile;
use super::types::OllamaEndpoint;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub(crate) fn inspect_target(
    target: &PreparedBundle,
    profile: &OllamaSpawnProfile,
) -> Result<(), TargetValidation> {
    if target.executable.path() != profile.executable().path()
        || target.executable.execution_identity() != profile.executable().execution_identity()
    {
        return Err(TargetValidation::InvalidTarget {
            code: OllamaErrorCode::OllamaBundleInvalid,
        });
    }
    let metadata =
        std::fs::metadata(target.executable.path()).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied => {
                TargetValidation::InvalidTarget {
                    code: OllamaErrorCode::OllamaBundleInvalid,
                }
            }
            _ => TargetValidation::Deferred {
                code: OllamaErrorCode::OllamaStorageUnavailable,
            },
        })?;
    if !metadata.is_file() {
        return Err(TargetValidation::InvalidTarget {
            code: OllamaErrorCode::OllamaBundleInvalid,
        });
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(TargetValidation::InvalidTarget {
            code: OllamaErrorCode::OllamaBundleInvalid,
        });
    }
    validate_executable_format(target.executable.path()).map_err(|error| match error {
        HashFileError::Missing | HashFileError::Permission | HashFileError::InvalidFormat => {
            TargetValidation::InvalidTarget {
                code: OllamaErrorCode::OllamaBundleInvalid,
            }
        }
        HashFileError::Other => TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaStorageUnavailable,
        },
    })?;
    let actual = hash_file(target.executable.path()).map_err(|error| match error {
        HashFileError::Missing | HashFileError::Permission | HashFileError::InvalidFormat => {
            TargetValidation::InvalidTarget {
                code: OllamaErrorCode::OllamaBundleInvalid,
            }
        }
        HashFileError::Other => TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaStorageUnavailable,
        },
    })?;
    if !target
        .fingerprint
        .executable_sha256
        .constant_time_eq(&actual)
    {
        return Err(TargetValidation::InvalidTarget {
            code: OllamaErrorCode::OllamaBundleInvalid,
        });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HttpProbeError {
    Cancelled,
    Deadline,
    Transport,
    Redirect,
    Status,
    Oversized,
    Malformed,
}

#[derive(Deserialize)]
struct VersionResponse {
    version: String,
}

pub(crate) async fn fetch_version(
    endpoint: &OllamaEndpoint,
    deadline: Instant,
    cancellation: &CancellationToken,
) -> Result<OllamaVersion, HttpProbeError> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .ok_or(HttpProbeError::Deadline)?;
    if cancellation.is_cancelled() {
        return Err(HttpProbeError::Cancelled);
    }
    let client = crate::services::secure_http::AuthenticatedClient::new_loopback(remaining)
        .map_err(|_| HttpProbeError::Transport)?;
    let request = client.get(format!("{}/api/version", endpoint.as_http_url()));
    let response = tokio::select! {
        _ = cancellation.cancelled() => return Err(HttpProbeError::Cancelled),
        _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
            return Err(HttpProbeError::Deadline);
        }
        result = client.send(request) => result.map_err(|error| match error {
            crate::services::secure_http::SecureHttpError::Redirect => HttpProbeError::Redirect,
            _ => HttpProbeError::Transport,
        })?,
    };
    if !response.status().is_success() {
        return Err(HttpProbeError::Status);
    }
    let body = tokio::select! {
        _ = cancellation.cancelled() => return Err(HttpProbeError::Cancelled),
        _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
            return Err(HttpProbeError::Deadline);
        }
        result = crate::services::secure_http::read_bounded(response, MAX_PROBE_RESPONSE_BYTES) => {
            result.map_err(|error| match error {
                crate::services::secure_http::SecureHttpError::BodyTooLarge => {
                    HttpProbeError::Oversized
                }
                _ => HttpProbeError::Transport,
            })?
        }
    };
    parse_version_body(&body)
}

pub(crate) fn parse_version_body(body: &[u8]) -> Result<OllamaVersion, HttpProbeError> {
    let response: VersionResponse =
        serde_json::from_slice(body).map_err(|_| HttpProbeError::Malformed)?;
    let normalized = response
        .version
        .strip_prefix('v')
        .unwrap_or(&response.version);
    OllamaVersion::parse(normalized).map_err(|_| HttpProbeError::Malformed)
}

#[derive(Clone, Copy)]
pub(crate) enum HashFileError {
    Missing,
    Permission,
    InvalidFormat,
    Other,
}

pub(crate) fn validate_executable_format(path: &Path) -> Result<(), HashFileError> {
    let mut file = std::fs::File::open(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => HashFileError::Missing,
        std::io::ErrorKind::PermissionDenied => HashFileError::Permission,
        _ => HashFileError::Other,
    })?;
    let mut header = [0_u8; 4096];
    let count = file.read(&mut header).map_err(|_| HashFileError::Other)?;
    let bytes = &header[..count];
    let valid = if bytes.starts_with(b"#!") {
        cfg!(unix)
    } else {
        #[cfg(target_os = "linux")]
        let native = bytes.starts_with(b"\x7fELF");
        #[cfg(target_os = "macos")]
        let native = matches!(
            bytes.get(..4),
            Some(b"\xfe\xed\xfa\xce")
                | Some(b"\xce\xfa\xed\xfe")
                | Some(b"\xfe\xed\xfa\xcf")
                | Some(b"\xcf\xfa\xed\xfe")
                | Some(b"\xca\xfe\xba\xbe")
                | Some(b"\xbe\xba\xfe\xca")
                | Some(b"\xca\xfe\xba\xbf")
                | Some(b"\xbf\xba\xfe\xca")
        );
        #[cfg(windows)]
        let native = bytes.starts_with(b"MZ")
            && bytes.len() >= 64
            && usize::try_from(u32::from_le_bytes([
                bytes[60], bytes[61], bytes[62], bytes[63],
            ]))
            .ok()
            .and_then(|offset| bytes.get(offset..offset.saturating_add(4)))
            .is_some_and(|signature| signature == b"PE\0\0");
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        let native = false;
        native
    };
    if valid {
        Ok(())
    } else {
        Err(HashFileError::InvalidFormat)
    }
}

pub(crate) fn hash_file(path: &Path) -> Result<Sha256Digest, HashFileError> {
    let mut file = std::fs::File::open(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => HashFileError::Missing,
        std::io::ErrorKind::PermissionDenied => HashFileError::Permission,
        _ => HashFileError::Other,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|_| HashFileError::Other)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Sha256Digest::from_hex(&hex::encode(hasher.finalize())).map_err(|_| HashFileError::Other)
}
