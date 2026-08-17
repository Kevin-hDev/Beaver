use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::durable_fs::{OllamaDurableFs, OllamaFsError, OllamaFsErrorKind};
use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion};
use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};
use super::probe_http::HashFileError;
use super::recovery_decision::DirectoryEvidence;
use super::spawn_profile_paths::active_executable;
use std::path::Path;

pub(super) fn fingerprint(fs: &dyn OllamaDurableFs, path: &Path) -> DirectoryEvidence {
    let identity = NativePathIdentityResolver;
    let root = match identity.canonical_directory(path) {
        Ok(root) => root,
        Err(code) => return classify_identity_failure("bundle-root-identity", code),
    };
    let executable_path = active_executable(root.path());
    if let Some(evidence) = classify_executable_shape(&executable_path) {
        return evidence;
    }
    let executable = match identity.canonical_executable(&executable_path) {
        Ok(executable) => executable,
        Err(code) => return classify_identity_failure("bundle-executable-identity", code),
    };
    let version_path = root.path().join("VERSION");
    let version_bytes = match fs.read_bounded(&version_path, MAX_DURABLE_DOCUMENT_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => return classify_version_read_failure(error),
    };
    let version = match std::str::from_utf8(&version_bytes)
        .ok()
        .map(str::trim)
        .and_then(|value| OllamaVersion::parse(value).ok())
    {
        Some(version) => version,
        None => {
            super::storage_error::record_classification("bundle-version-parse", "invalid");
            return DirectoryEvidence::Invalid;
        }
    };
    let digest = match super::probe_http::hash_file(executable.path()) {
        Ok(digest) => digest,
        Err(error) => return classify_hash_failure(error),
    };
    DirectoryEvidence::Present(BundleFingerprint {
        version,
        executable_sha256: digest,
    })
}

fn classify_executable_shape(path: &Path) -> Option<DirectoryEvidence> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => None,
        Ok(_) => Some(DirectoryEvidence::Invalid),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Some(DirectoryEvidence::Invalid)
        }
        Err(error) => {
            super::storage_error::record_io("bundle-executable-inspect", &error);
            Some(DirectoryEvidence::Unknown)
        }
    }
}

fn classify_identity_failure(context: &'static str, code: OllamaErrorCode) -> DirectoryEvidence {
    super::storage_error::record_classification(context, code);
    match code {
        OllamaErrorCode::OllamaModelStoreConflict => DirectoryEvidence::Invalid,
        _ => DirectoryEvidence::Unknown,
    }
}

fn classify_version_read_failure(error: OllamaFsError) -> DirectoryEvidence {
    super::storage_error::record_durable("bundle-version-read", error);
    match error.kind() {
        OllamaFsErrorKind::NotFound | OllamaFsErrorKind::InvalidInput => DirectoryEvidence::Invalid,
        _ => DirectoryEvidence::Unknown,
    }
}

fn classify_hash_failure(error: HashFileError) -> DirectoryEvidence {
    super::storage_error::record_classification("bundle-executable-hash", error.diagnostic());
    match error {
        HashFileError::Missing | HashFileError::InvalidFormat => DirectoryEvidence::Invalid,
        HashFileError::Permission | HashFileError::Other => DirectoryEvidence::Unknown,
    }
}
