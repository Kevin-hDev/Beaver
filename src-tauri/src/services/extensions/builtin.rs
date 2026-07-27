use super::types::{ExtensionKind, ExtensionRecord};

pub fn records() -> Vec<ExtensionRecord> {
    Vec::new()
}

pub fn merge(mut stored: Vec<ExtensionRecord>) -> Vec<ExtensionRecord> {
    let mut merged = records();
    stored.retain(|item| item.kind != ExtensionKind::Builtin);
    merged.extend(stored);
    merged
}
