//! Codes d'erreur du domaine extensions, traduits côté interface.
//! Chaque code ajouté ici doit être déclaré dans `EXTENSION_BACKEND_ERROR_CODES`
//! (`src/lib/extension-errors.ts`) et traduit dans les sept langues.

pub const HOST_UNAVAILABLE: &str = "extensions_host_unavailable";
pub const HOST_BUSY: &str = "extensions_host_busy";
pub const HOST_TIMEOUT: &str = "extensions_host_timeout";
pub const REQUEST_TOO_LARGE: &str = "extensions_request_too_large";
pub const REQUEST_INVALID: &str = "extensions_request_invalid";
pub const TOOL_UNAVAILABLE: &str = "extensions_tool_unavailable";
pub const TOOL_ARGUMENTS_INVALID: &str = "extensions_tool_arguments_invalid";
pub const BUILTIN_CATALOG_INVALID: &str = "extensions_builtin_catalog_invalid";
pub const BUILTIN_CATALOG_UNAVAILABLE: &str = "extensions_builtin_catalog_unavailable";
pub const BUILTIN_PLUGIN_INVALID: &str = "extensions_builtin_plugin_invalid";
pub const BUILTIN_ENTRY_MISSING: &str = "extensions_builtin_entry_missing";
pub const BUILTIN_ENTRY_UNAVAILABLE: &str = "extensions_builtin_entry_unavailable";
pub const BUILTIN_ENTRY_INVALID: &str = "extensions_builtin_entry_invalid";

/// Sert au test qui exige une traduction par code et par langue.
#[cfg(test)]
pub const ALL: &[&str] = &[
    HOST_UNAVAILABLE,
    HOST_BUSY,
    HOST_TIMEOUT,
    REQUEST_TOO_LARGE,
    REQUEST_INVALID,
    TOOL_UNAVAILABLE,
    TOOL_ARGUMENTS_INVALID,
    BUILTIN_CATALOG_INVALID,
    BUILTIN_CATALOG_UNAVAILABLE,
    BUILTIN_PLUGIN_INVALID,
    BUILTIN_ENTRY_MISSING,
    BUILTIN_ENTRY_UNAVAILABLE,
    BUILTIN_ENTRY_INVALID,
];

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
}
