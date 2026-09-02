//! Codes d'erreur du domaine Extensions générés depuis le contrat exécutable.

pub use super::types::backend_error_codes::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::{BTreeSet, HashSet};
    use std::path::Path;

    #[test]
    fn every_error_code_has_exactly_one_translation_per_locale() {
        let expected = ALL.iter().copied().collect::<BTreeSet<_>>();
        let translations = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/i18n");

        for locale in ["fr", "en", "es", "de", "it", "zh", "ja"] {
            let path = translations.join(format!("{locale}.json"));
            let raw = std::fs::read_to_string(path).expect("translation file must be readable");
            let document: Value =
                serde_json::from_str(&raw).expect("translation JSON must be valid");
            let codes = document
                .pointer("/extensions/errors/codes")
                .and_then(Value::as_object)
                .expect("extension error translations must exist");
            let actual = codes.keys().map(String::as_str).collect::<BTreeSet<_>>();

            assert_eq!(actual, expected, "error translations differ for {locale}");
            assert!(
                codes.values().all(|translation| translation
                    .as_str()
                    .is_some_and(|text| !text.trim().is_empty())),
                "empty error translation for {locale}"
            );
        }
    }

    #[test]
    fn every_code_is_unique_and_shaped_for_translation() {
        let mut seen = HashSet::with_capacity(ALL.len());
        for code in ALL {
            assert!(seen.insert(*code), "code dupliqué: {code}");
            assert!(code.starts_with("extensions_"), "préfixe manquant: {code}");
            assert!(
                code.chars()
                    .all(|character| character.is_ascii_lowercase() || character == '_'),
                "forme invalide: {code}"
            );
        }
    }

    #[test]
    fn malformed_host_payload_uses_the_incompatible_host_code() {
        let error = super::super::runtime::parse::<Vec<String>>(serde_json::Value::String(
            "invalid payload".to_string(),
        ))
        .unwrap_err();

        assert_eq!(error, HOST_INCOMPATIBLE);
    }
}
