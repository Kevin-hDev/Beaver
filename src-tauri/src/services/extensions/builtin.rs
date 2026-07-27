use super::types::{ExtensionKind, ExtensionRecord, ExtensionStatus};

pub fn records() -> Vec<ExtensionRecord> {
    Vec::new()
}

pub fn merge(mut stored: Vec<ExtensionRecord>) -> Vec<ExtensionRecord> {
    let mut merged = records();
    stored.retain(|item| item.kind != ExtensionKind::Builtin);
    for record in &mut stored {
        if record.enabled {
            record.status = ExtensionStatus::Inactive;
        }
    }
    merged.extend(stored);
    merged
}
