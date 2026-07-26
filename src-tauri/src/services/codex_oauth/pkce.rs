use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub struct PkceChallenge {
    pub verifier: Zeroizing<String>,
    pub challenge: String,
}

pub fn generate() -> PkceChallenge {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    bytes.fill(0);
    let hash = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hash);
    PkceChallenge {
        verifier: Zeroizing::new(verifier),
        challenge,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_pair() {
        let p = generate();
        assert_eq!(p.verifier.len(), 43);
        assert_eq!(p.challenge.len(), 43);
        assert_ne!(p.verifier.as_str(), p.challenge.as_str());
        assert!(p
            .verifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn challenge_matches_verifier() {
        let p = generate();
        let hash = Sha256::digest(p.verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(hash);
        assert_eq!(p.challenge, expected);
    }
}
