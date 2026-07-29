//! Codes d'erreur du domaine extensions, traduits côté interface.
//! Chaque code ajouté ici doit être déclaré dans `EXTENSION_BACKEND_ERROR_CODES`
//! (`src/lib/extension-errors.ts`) et traduit dans les sept langues.

macro_rules! declare_error_codes {
    ($($name:ident => $value:literal),+ $(,)?) => {
        $(pub const $name: &str = $value;)+
        #[cfg(test)]
        pub const ALL: &[&str] = &[$($name),+];
    };
}

declare_error_codes! {
    HOST_UNAVAILABLE => "extensions_host_unavailable",
    HOST_BUSY => "extensions_host_busy",
    HOST_TIMEOUT => "extensions_host_timeout",
    REQUEST_TOO_LARGE => "extensions_request_too_large",
    REQUEST_INVALID => "extensions_request_invalid",
    TOOL_UNAVAILABLE => "extensions_tool_unavailable",
    TOOL_ARGUMENTS_INVALID => "extensions_tool_arguments_invalid",
    BUILTIN_CATALOG_INVALID => "extensions_builtin_catalog_invalid",
    BUILTIN_CATALOG_UNAVAILABLE => "extensions_builtin_catalog_unavailable",
    BUILTIN_PLUGIN_INVALID => "extensions_builtin_plugin_invalid",
    BUILTIN_ENTRY_MISSING => "extensions_builtin_entry_missing",
    BUILTIN_ENTRY_UNAVAILABLE => "extensions_builtin_entry_unavailable",
    BUILTIN_ENTRY_INVALID => "extensions_builtin_entry_invalid",
    INSTALL_FAILED => "extensions_install_failed",
    UPDATE_FAILED => "extensions_update_failed",
    UNINSTALL_FAILED => "extensions_uninstall_failed",
    SOURCE_INVALID => "extensions_source_invalid",
    PACKAGE_INVALID => "extensions_package_invalid",
    GIT_DOWNLOAD_FAILED => "extensions_git_download_failed",
    GIT_TIMEOUT => "extensions_git_timeout",
    RUNTIME_UNAVAILABLE => "extensions_runtime_unavailable",
    ENVIRONMENT_INVALID => "extensions_environment_invalid",
    DEPENDENCY_INSTALL_FAILED => "extensions_dependency_install_failed",
    MANIFEST_INVALID => "extensions_manifest_invalid",
    NOT_BEAVER_EXTENSION => "extensions_not_beaver_extension",
    API_INCOMPATIBLE => "extensions_api_incompatible",
    SYMLINK_UNSUPPORTED => "extensions_symlink_unsupported",
    ALREADY_INSTALLED => "extensions_already_installed",
    LIMIT_REACHED => "extensions_limit_reached",
    STORAGE_FAILED => "extensions_storage_failed",
    UPDATE_IDENTITY_CHANGED => "extensions_update_identity_changed",
    UPDATE_UNAVAILABLE => "extensions_update_unavailable",
    CLEANUP_FAILED => "extensions_cleanup_failed",
}

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
