use std::collections::BTreeMap;

#[path = "app_update_notes_wire.rs"]
mod wire;

const SUPPORTED_LOCALES: [&str; 7] = ["fr", "en", "es", "de", "it", "zh", "ja"];
const MAX_VERSION_ENTRIES: usize = 50;
const MAX_BULLETS: usize = 6;
const MAX_BULLET_CHARS: usize = 180;
pub(crate) const MAX_RELEASE_NOTES_BYTES: usize = 64 * 1024;

pub(crate) type AppReleaseNotesByLocale = BTreeMap<String, Vec<String>>;

pub(crate) fn parse_app_release_notes_json(
    bytes: &[u8],
    version: &str,
) -> Option<AppReleaseNotesByLocale> {
    if bytes.len() > MAX_RELEASE_NOTES_BYTES {
        return None;
    }

    let notes = wire::parse(bytes, version)?;
    validate_release_notes(&notes)
}

fn validate_release_notes(notes: &AppReleaseNotesByLocale) -> Option<AppReleaseNotesByLocale> {
    if notes.len() != SUPPORTED_LOCALES.len() {
        return None;
    }

    let mut out = BTreeMap::new();
    for locale in SUPPORTED_LOCALES {
        let items = notes.get(locale)?;
        if items.is_empty() || items.len() > MAX_BULLETS {
            return None;
        }

        let mut clean_items = Vec::with_capacity(items.len());
        for item in items {
            let clean = validate_note_item(item)?;
            clean_items.push(clean);
        }
        out.insert(locale.to_string(), clean_items);
    }

    Some(out)
}

fn validate_note_item(item: &str) -> Option<String> {
    let trimmed = item.trim();
    if trimmed.is_empty() || trimmed != item {
        return None;
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return None;
    }
    if trimmed.chars().count() > MAX_BULLET_CHARS {
        return None;
    }
    if !is_complete_sentence(trimmed) {
        return None;
    }
    Some(trimmed.to_string())
}

fn is_complete_sentence(item: &str) -> bool {
    matches!(
        item.chars().last(),
        Some('.') | Some('!') | Some('?') | Some('。') | Some('！') | Some('？')
    )
}

#[cfg(test)]
#[path = "app_update_notes_tests.rs"]
mod tests;
