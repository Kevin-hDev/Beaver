use super::types::{ExtensionContributions, ExtensionKind, ExtensionRecord, ExtensionStatus};

pub fn reset_hosted_runtime(mut records: Vec<ExtensionRecord>) -> Vec<ExtensionRecord> {
    for record in &mut records {
        if record.kind != ExtensionKind::External {
            if record.kind == ExtensionKind::Local && !record.trusted {
                record.enabled = false;
            }
            record.status = ExtensionStatus::Inactive;
            record.last_error = None;
            record.contributions = ExtensionContributions::default();
        }
    }
    records
}
