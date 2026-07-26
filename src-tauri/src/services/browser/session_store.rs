use super::session_model::SessionModel;
use std::path::{Path, PathBuf};
use zeroize::{Zeroize, Zeroizing};

pub(super) const MAX_SESSION_FILE_BYTES: usize = 128 * 1024;
const MAX_SESSION_PLAINTEXT_BYTES: usize = 64 * 1024;
const SESSION_KEY_NAME: &str = "browser-session-key-v1";

pub(super) fn sessions_dir() -> PathBuf {
    crate::services::paths::data_dir()
        .join("browser")
        .join("sessions")
}

pub(super) fn session_key() -> Result<Zeroizing<Vec<u8>>, ()> {
    crate::services::api_keys::get_or_create_random_raw(SESSION_KEY_NAME, 32).map_err(|_| ())
}

pub(super) fn load_at(
    directory: &Path,
    session_id: &str,
    key: &[u8],
) -> Result<Option<SessionModel>, ()> {
    let path = session_path(directory, session_id)?;
    if !path.exists() {
        return Ok(None);
    }
    let metadata = std::fs::metadata(&path).map_err(|_| ())?;
    if metadata.len() > MAX_SESSION_FILE_BYTES as u64 {
        return Err(());
    }
    let encrypted = std::fs::read(path).map_err(|_| ())?;
    if encrypted.len() > MAX_SESSION_FILE_BYTES {
        return Err(());
    }
    let mut plaintext = crate::services::vault::decrypt(key, &encrypted).map_err(|_| ())?;
    if plaintext.len() > MAX_SESSION_PLAINTEXT_BYTES {
        plaintext.zeroize();
        return Err(());
    }
    let result = SessionModel::restore(&plaintext);
    plaintext.zeroize();
    result.map(Some)
}

pub(super) fn save_at(
    directory: &Path,
    session_id: &str,
    key: &[u8],
    model: &SessionModel,
) -> Result<(), ()> {
    let path = session_path(directory, session_id)?;
    let mut plaintext = serde_json::to_vec(&model.persisted()).map_err(|_| ())?;
    if plaintext.len() > MAX_SESSION_PLAINTEXT_BYTES {
        plaintext.zeroize();
        return Err(());
    }
    let encrypted = crate::services::vault::encrypt(key, &plaintext).map_err(|_| ());
    plaintext.zeroize();
    let encrypted = encrypted?;
    if encrypted.len() > MAX_SESSION_FILE_BYTES {
        return Err(());
    }
    crate::services::private_store::atomic_write(&path, &encrypted).map_err(|_| ())
}

fn session_path(directory: &Path, session_id: &str) -> Result<PathBuf, ()> {
    crate::services::agent_local::session_store::validate_session_id(session_id).map_err(|_| ())?;
    Ok(directory.join(format!("{session_id}.enc")))
}
