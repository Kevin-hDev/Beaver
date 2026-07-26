use super::*;

// Clé master 32 octets (XChaCha20-Poly1305) pour les tests.
fn test_key() -> Vec<u8> {
    let mut k = vec![0u8; 32];
    for (i, b) in k.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(7);
    }
    k
}

#[test]
fn round_trip_recovers_exact_plaintext() {
    let key = test_key();
    let plaintext = br#"{"groq":"sk-test-123456789"}"#;

    let encrypted = encrypt(&key, plaintext).expect("encrypt should succeed");
    let decrypted = decrypt(&key, &encrypted).expect("decrypt should succeed");

    assert_eq!(decrypted.as_slice(), &plaintext[..]);
}

#[test]
fn nonce_is_random_ciphertext_differs_each_call() {
    // AEAD avec nonce aléatoire : chiffrer 2x le même contenu donne 2
    // ciphertexts différents. Si ce test échoue, le nonce est réutilisé
    // (catastrophique en sécurité — on peut forger des messages).
    let key = test_key();
    let plaintext = b"same secret payload";

    let c1 = encrypt(&key, plaintext).expect("encrypt 1");
    let c2 = encrypt(&key, plaintext).expect("encrypt 2");

    assert_ne!(
        c1, c2,
        "le nonce doit être régénéré à chaque appel (pas de réutilisation)"
    );
}

#[test]
fn decrypt_fails_with_wrong_key() {
    // Fail CLOSED : une mauvaise clé doit échouer (tag Poly1305 invalide),
    // jamais retourner du pseudo-plaintext silencieux.
    let key = test_key();
    let mut wrong_key = vec![0u8; 32];
    wrong_key.fill(0xFF);

    let plaintext = b"top secret api key";
    let encrypted = encrypt(&key, plaintext).expect("encrypt");

    let result = decrypt(&wrong_key, &encrypted);
    assert!(
        result.is_err(),
        "une mauvaise clé doit échouer (échec attendu, pas de déchiffrement partiel)"
    );
}

#[test]
fn decrypt_rejects_tampered_ciphertext() {
    // Modifier un octet du fichier chiffré doit invalider le tag
    // d'authentification Poly1305.
    let key = test_key();
    let plaintext = b"integrity matters";
    let mut encrypted = encrypt(&key, plaintext).expect("encrypt");

    // Corrompre un octet au milieu du payload JSON (éviter l'en-tête).
    let mid = encrypted.len() / 2;
    encrypted[mid] ^= 0xFF;

    let result = decrypt(&key, &encrypted);
    assert!(result.is_err(), "un ciphertext altéré doit être rejeté");
}

#[test]
fn decrypt_rejects_unsupported_version() {
    let key = test_key();
    let plaintext = b"versioned payload";
    let mut encrypted = encrypt(&key, plaintext).expect("encrypt");

    // Le 1er champ du JSON sérialisé est "version":1. On remplace la valeur
    // par une version inconnue (99). La chaîne "version": 1 est présente en
    // tête du flux serde_json::to_vec_pretty.
    let needle = b"\"version\": 1";
    if let Some(pos) = encrypted.windows(needle.len()).position(|w| w == needle) {
        encrypted[pos + needle.len() - 1] = b'9'; // version 9
        encrypted.insert(pos + needle.len(), b'9'); // 99
    }

    let result = decrypt(&key, &encrypted);
    assert!(
        result.is_err(),
        "une version de vault non supportée doit être rejetée"
    );
}

#[test]
fn decrypt_rejects_invalid_json() {
    let key = test_key();
    let garbage = b"this is not json at all";

    let result = decrypt(&key, garbage);
    assert!(result.is_err(), "du contenu non-JSON doit être rejeté");
}

#[test]
fn vault_file_round_trips_through_serde() {
    // La structure de persistance doit survivre à un cycle
    // serialize -> deserialize sans perte.
    let original = VaultFile {
        version: VAULT_VERSION,
        nonce: B64.encode(b"dummy-nonce-24-bytes-here!"),
        data: B64.encode(b"ciphertext"),
    };

    let bytes = serde_json::to_vec(&original).expect("serialize");
    let restored: VaultFile = serde_json::from_slice(&bytes).expect("deserialize");

    assert_eq!(restored.version, original.version);
    assert_eq!(restored.nonce, original.nonce);
    assert_eq!(restored.data, original.data);
}

#[test]
fn encrypt_rejects_wrong_key_length() {
    // XChaCha20 exige une clé de 32 octets. Une clé trop courte doit échouer
    // explicitement (fail closed) plutôt que de tronquer/padder.
    let short_key = vec![0u8; 16];
    let result = encrypt(&short_key, b"data");
    assert!(result.is_err(), "une clé trop courte doit être rejetée");
}

#[test]
fn decrypt_rejects_nonce_with_wrong_length_without_panicking() {
    for nonce in [vec![0_u8; 23], vec![0_u8; 25]] {
        let malformed = VaultFile {
            version: VAULT_VERSION,
            nonce: B64.encode(nonce),
            data: B64.encode(b"ciphertext"),
        };
        let bytes = serde_json::to_vec(&malformed).unwrap();
        assert!(decrypt(&test_key(), &bytes).is_err());
    }
}
