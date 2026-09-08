use super::session_model::SessionModel;
use std::path::{Path, PathBuf};
#[cfg(any(feature = "e2e", test))]
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

pub(super) const MAX_SESSION_FILE_BYTES: usize = 128 * 1024;
const MAX_SESSION_PLAINTEXT_BYTES: usize = 64 * 1024;
const SESSION_KEY_NAME: &str = "browser-session-key-v1";
pub(super) const SESSION_KEY_BYTES: usize = 32;
#[cfg(feature = "e2e")]
const E2E_SESSION_KEY_ENV: &str = "BEAVER_E2E_BROWSER_SESSION_KEY_B64";
#[cfg(any(feature = "e2e", test))]
const E2E_SESSION_KEY_BASE64_BYTES: usize = SESSION_KEY_BYTES.div_ceil(3) * 4;

#[cfg(any(feature = "e2e", test))]
pub(super) enum E2eSessionKeyAction {
    Seed,
    Keep,
}

pub(super) fn sessions_dir() -> PathBuf {
    crate::services::paths::data_dir()
        .join("browser")
        .join("sessions")
}

pub(super) fn session_key() -> Result<Zeroizing<Vec<u8>>, ()> {
    #[cfg(feature = "e2e")]
    seed_e2e_session_key_if_configured()?;

    crate::services::api_keys::get_or_create_random_raw(SESSION_KEY_NAME, SESSION_KEY_BYTES)
        .map_err(|_| ())
}

#[cfg(feature = "e2e")]
pub(super) fn seed_e2e_session_key_fixture() -> Result<(), ()> {
    let encoded = e2e_session_key_from_env()?.ok_or(())?;
    seed_e2e_session_key(&encoded)
}

#[cfg(feature = "e2e")]
fn seed_e2e_session_key_if_configured() -> Result<(), ()> {
    let Some(encoded) = e2e_session_key_from_env()? else {
        return Ok(());
    };
    seed_e2e_session_key(&encoded)
}

#[cfg(feature = "e2e")]
fn seed_e2e_session_key(encoded: &str) -> Result<(), ()> {
    let exists = crate::services::api_keys::has_raw(SESSION_KEY_NAME).map_err(|_| ())?;
    let existing = exists
        .then(|| crate::services::api_keys::get_raw(SESSION_KEY_NAME))
        .transpose()
        .map_err(|_| ())?;

    // E2E only: capability checks can request session_key before the WebView bridge.
    // Seed here so the runner-owned key wins before random generation can occur.
    match e2e_session_key_action(existing.as_deref().map(String::as_str), encoded)? {
        E2eSessionKeyAction::Seed => {
            crate::services::api_keys::set_raw(SESSION_KEY_NAME, encoded).map_err(|_| ())
        }
        E2eSessionKeyAction::Keep => Ok(()),
    }
}

#[cfg(feature = "e2e")]
fn e2e_session_key_from_env() -> Result<Option<Zeroizing<String>>, ()> {
    match std::env::var(E2E_SESSION_KEY_ENV) {
        Ok(value) => Ok(Some(Zeroizing::new(value))),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(()),
    }
}

#[cfg(any(feature = "e2e", test))]
pub(super) fn e2e_session_key_action(
    existing: Option<&str>,
    supplied: &str,
) -> Result<E2eSessionKeyAction, ()> {
    let supplied = decode_e2e_session_key(supplied)?;
    let Some(existing) = existing else {
        return Ok(E2eSessionKeyAction::Seed);
    };
    let existing = decode_e2e_session_key(existing)?;
    let same: bool = supplied.as_slice().ct_eq(existing.as_slice()).into();
    same.then_some(E2eSessionKeyAction::Keep).ok_or(())
}

#[cfg(any(feature = "e2e", test))]
fn decode_e2e_session_key(encoded: &str) -> Result<Zeroizing<Vec<u8>>, ()> {
    use base64::Engine;

    if encoded.len() != E2E_SESSION_KEY_BASE64_BYTES {
        return Err(());
    }
    let decoded = Zeroizing::new(
        base64::engine::general_purpose::STANDARD
            .decode(encoded.as_bytes())
            .map_err(|_| ())?,
    );
    if decoded.len() != SESSION_KEY_BYTES {
        return Err(());
    }
    let canonical = Zeroizing::new(base64::engine::general_purpose::STANDARD.encode(&*decoded));
    let valid: bool = canonical.as_bytes().ct_eq(encoded.as_bytes()).into();
    valid.then_some(decoded).ok_or(())
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
