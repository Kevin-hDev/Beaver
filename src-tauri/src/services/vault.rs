use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use keyring::Entry;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "cl-go-dash";
const MASTER_KEY_USER: &str = "master-key";
const VAULT_VERSION: u8 = 1;

const KNOWN_PROVIDERS: &[&str] = &[
    "groq",
    "google",
    "mistral",
    "cerebras",
    "openrouter",
    "openai",
    "deepseek",
    "brave",
    "exa",
    "firecrawl",
];

#[derive(Serialize, Deserialize)]
pub(crate) struct VaultFile {
    pub(crate) version: u8,
    pub(crate) nonce: String,
    pub(crate) data: String,
}

pub fn vault_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("secrets.enc")
}

#[cfg(test)]
#[path = "vault_tests.rs"]
pub(crate) mod tests;

pub fn load_or_create_master_key() -> Result<Zeroizing<Vec<u8>>, String> {
    let entry =
        Entry::new(KEYRING_SERVICE, MASTER_KEY_USER).map_err(|e| format!("keyring entry: {e}"))?;

    match entry.get_password() {
        Ok(b64_raw) => {
            let b64 = Zeroizing::new(b64_raw);
            let mut bytes = B64
                .decode(b64.as_str())
                .map_err(|e| format!("master key decode: {e}"))?;
            if bytes.len() == 32 {
                return Ok(Zeroizing::new(bytes));
            }
            bytes.zeroize();
            Err("master key in keychain has invalid length".to_string())
        }
        Err(keyring::Error::NoEntry) => {
            if vault_path().exists() {
                return Err(
                    "master key missing from keychain but vault exists — cannot decrypt"
                        .to_string(),
                );
            }
            let mut key = vec![0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            let mut b64 = B64.encode(&key);
            let set_result = entry.set_password(&b64);
            b64.zeroize();
            if let Err(e) = set_result {
                key.zeroize();
                return Err(format!("keyring set master: {e}"));
            }
            Ok(Zeroizing::new(key))
        }
        Err(e) => Err(format!("keychain access failed: {e}")),
    }
}

pub(crate) fn encrypt(master_key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher =
        XChaCha20Poly1305::new_from_slice(master_key).map_err(|e| format!("cipher init: {e}"))?;

    let mut nonce_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("encrypt: {e}"))?;

    let vault_file = VaultFile {
        version: VAULT_VERSION,
        nonce: B64.encode(nonce_bytes),
        data: B64.encode(&ciphertext),
    };
    serde_json::to_vec_pretty(&vault_file).map_err(|e| format!("json: {e}"))
}

pub(crate) fn decrypt(master_key: &[u8], vault_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let vf: VaultFile =
        serde_json::from_slice(vault_bytes).map_err(|e| format!("vault parse: {e}"))?;
    if vf.version != VAULT_VERSION {
        return Err(format!("unsupported vault version: {}", vf.version));
    }

    let nonce_bytes = B64
        .decode(&vf.nonce)
        .map_err(|e| format!("nonce decode: {e}"))?;
    let nonce_array: [u8; 24] = nonce_bytes
        .try_into()
        .map_err(|_| "vault nonce invalide".to_string())?;
    let ciphertext = B64
        .decode(&vf.data)
        .map_err(|e| format!("data decode: {e}"))?;

    let nonce = XNonce::from(nonce_array);
    let cipher =
        XChaCha20Poly1305::new_from_slice(master_key).map_err(|e| format!("cipher init: {e}"))?;

    cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|e| format!("decrypt: {e}"))
}

pub fn read_vault(master_key: &[u8]) -> Result<HashMap<String, String>, String> {
    let path = vault_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("read vault: {e}"))?;
    let mut plaintext = decrypt(master_key, &bytes)?;
    let result = serde_json::from_slice(&plaintext).map_err(|e| format!("vault json: {e}"));
    plaintext.zeroize();
    result
}

pub fn write_vault(master_key: &[u8], map: &HashMap<String, String>) -> Result<(), String> {
    let path = vault_path();
    let mut plaintext = serde_json::to_vec(map).map_err(|e| format!("json: {e}"))?;
    let encrypt_result = encrypt(master_key, &plaintext);
    plaintext.zeroize();
    let encrypted = encrypt_result?;

    crate::services::private_store::atomic_write(&path, &encrypted)
}

pub fn read_legacy_keychain_keys() -> HashMap<String, Zeroizing<String>> {
    let mut found = HashMap::new();
    if let Ok(entry) = Entry::new(KEYRING_SERVICE, "brave_api_key") {
        if let Ok(key) = entry.get_password() {
            found.insert("brave".to_string(), Zeroizing::new(key));
        }
    }
    for id in KNOWN_PROVIDERS {
        if found.contains_key(*id) {
            continue;
        }
        let Ok(entry) = Entry::new(KEYRING_SERVICE, id) else {
            continue;
        };
        if let Ok(key) = entry.get_password() {
            found.insert(id.to_string(), Zeroizing::new(key));
        }
    }
    found
}
