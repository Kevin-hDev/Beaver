use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::Zeroizing;

const VAULT_KEY: &str = "reasoning.diagnostics.hmac.v1";
const DOMAIN: &[u8] = b"beaver-reasoning-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FingerprintContext<'a> {
    pub session_id: &'a str,
    pub turn_id: &'a str,
    pub contract_id: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintError {
    Unavailable,
}

pub fn opaque_hmac(
    context: FingerprintContext<'_>,
    opaque: &[u8],
) -> Result<String, FingerprintError> {
    let key = crate::services::api_keys::get_or_create_random_raw(VAULT_KEY, 32)
        .map_err(|_| FingerprintError::Unavailable)?;
    Ok(opaque_hmac_with_key(&key, context, opaque))
}

pub(crate) fn ensure_fingerprint_key() -> Result<(), FingerprintError> {
    crate::services::api_keys::get_or_create_random_raw(VAULT_KEY, 32)
        .map(|_| ())
        .map_err(|_| FingerprintError::Unavailable)
}

pub(crate) fn opaque_hmac_with_key(
    key: &Zeroizing<Vec<u8>>,
    context: FingerprintContext<'_>,
    opaque: &[u8],
) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("32 byte HMAC key");
    mac.update(DOMAIN);
    for value in [
        context.session_id.as_bytes(),
        context.turn_id.as_bytes(),
        context.contract_id.as_bytes(),
        opaque,
    ] {
        mac.update(&(value.len() as u64).to_be_bytes());
        mac.update(value);
    }
    hex::encode(mac.finalize().into_bytes())
}
