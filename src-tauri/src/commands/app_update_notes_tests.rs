use super::*;

fn valid_notes_json(version: &str) -> String {
    format!(
        r#"{{
  "{version}": {{
    "fr": ["Note française complète."],
    "en": ["Complete English note."],
    "es": ["Nota española completa."],
    "de": ["Vollständige deutsche Notiz."],
    "it": ["Nota italiana completa."],
    "zh": ["完整的中文说明。"],
    "ja": ["完全な日本語の説明です。"]
  }}
}}"#
    )
}

#[test]
fn parses_release_notes_for_matching_version() {
    let notes =
        parse_app_release_notes_json(valid_notes_json("0.9.4").as_bytes(), "0.9.4").expect("notes");

    assert_eq!(notes["en"], vec!["Complete English note."]);
    assert_eq!(notes["fr"], vec!["Note française complète."]);
}

#[test]
fn accepts_v_prefixed_version_keys() {
    let notes = parse_app_release_notes_json(valid_notes_json("v0.9.4").as_bytes(), "0.9.4")
        .expect("notes");

    assert_eq!(notes["ja"], vec!["完全な日本語の説明です。"]);
}

#[test]
fn rejects_missing_locale() {
    let json = r#"{
      "0.9.4": {
        "en": ["Complete English note."]
      }
    }"#;

    assert!(parse_app_release_notes_json(json.as_bytes(), "0.9.4").is_none());
}

#[test]
fn rejects_overlong_notes_without_truncating() {
    let json = valid_notes_json("0.9.4").replace(
        "Complete English note.",
        &format!("{}.", "x".repeat(MAX_BULLET_CHARS + 1)),
    );

    assert!(parse_app_release_notes_json(json.as_bytes(), "0.9.4").is_none());
}

#[test]
fn rejects_incomplete_sentences() {
    let json =
        valid_notes_json("0.9.4").replace("Complete English note.", "Incomplete English note");

    assert!(parse_app_release_notes_json(json.as_bytes(), "0.9.4").is_none());
}

#[test]
fn rejects_large_payloads() {
    let bytes = vec![b' '; MAX_RELEASE_NOTES_BYTES + 1];

    assert!(parse_app_release_notes_json(&bytes, "0.9.4").is_none());
}

#[test]
fn rejects_duplicate_version_and_locale_keys() {
    let entry = valid_notes_json("0.9.4");
    let entry = entry
        .trim()
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap();
    let duplicate_version = format!("{{{entry},{entry}}}");
    assert!(parse_app_release_notes_json(duplicate_version.as_bytes(), "0.9.4").is_none());

    let duplicate_locale = valid_notes_json("0.9.4").replace(
        r#""en": ["Complete English note."],"#,
        r#""en": ["Complete English note."], "en": ["Other complete note."],"#,
    );
    assert!(parse_app_release_notes_json(duplicate_locale.as_bytes(), "0.9.4").is_none());
}

#[test]
fn duplicate_noise_entries_cannot_bypass_the_version_limit() {
    let valid = valid_notes_json("0.9.4");
    let locale_block = valid
        .split_once(r#""0.9.4": "#)
        .and_then(|(_, value)| value.trim().strip_suffix('}'))
        .unwrap();
    let mut entries = Vec::with_capacity(MAX_VERSION_ENTRIES + 1);
    entries.push(format!(r#""0.9.4": {locale_block}"#));
    for _ in 0..MAX_VERSION_ENTRIES {
        entries.push(format!(r#""noise": {locale_block}"#));
    }
    let payload = format!("{{{}}}", entries.join(","));

    assert!(parse_app_release_notes_json(payload.as_bytes(), "0.9.4").is_none());
}
