use crate::services::extensions::types::DIAGNOSTIC_CODES;
use crate::services::extensions::ui_contract::UI_DIAGNOSTIC_CODES;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn every_contract_diagnostic_has_exactly_one_translation_per_locale() {
    let expected = DIAGNOSTIC_CODES
        .iter()
        .chain(UI_DIAGNOSTIC_CODES.iter())
        .copied()
        .collect::<BTreeSet<_>>();
    let translations = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/i18n");

    for locale in ["fr", "en", "es", "de", "it", "zh", "ja"] {
        let path = translations.join(format!("{locale}.json"));
        let raw = std::fs::read_to_string(path).expect("translation file must be readable");
        let document: Value = serde_json::from_str(&raw).expect("translation JSON must be valid");
        let codes = document
            .pointer("/extensions/diagnostics/codes")
            .and_then(Value::as_object)
            .expect("extension diagnostic translations must exist");
        let actual = codes.keys().map(String::as_str).collect::<BTreeSet<_>>();

        assert_eq!(
            actual, expected,
            "diagnostic translations differ for {locale}"
        );
        assert!(
            codes.values().all(|translation| translation
                .as_str()
                .is_some_and(|text| !text.trim().is_empty())),
            "empty diagnostic translation for {locale}"
        );
    }
}
