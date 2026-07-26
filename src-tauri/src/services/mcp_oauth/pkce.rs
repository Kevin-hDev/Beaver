use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64URL, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const VERIFIER_LEN: usize = 64;

pub fn generate() -> (Zeroizing<String>, String) {
    let mut bytes = [0u8; VERIFIER_LEN];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let verifier = Zeroizing::new(B64URL.encode(bytes));
    bytes.fill(0);

    let hash = Sha256::digest(verifier.as_bytes());
    let challenge = B64URL.encode(hash);

    (verifier, challenge)
}
