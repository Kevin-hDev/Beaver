use super::types::{ExtensionContributions, ExtensionRecord};
use serde::Serialize;

/// Projection envoyée à l'interface.
///
/// `ExtensionRecord` omet les contributions lors de sa sérialisation afin de
/// ne jamais les persister. Cette projection les réintroduit uniquement pour
/// les réponses IPC.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionView {
    #[serde(flatten)]
    record: ExtensionRecord,
    contributions: ExtensionContributions,
}

impl From<ExtensionRecord> for ExtensionView {
    fn from(record: ExtensionRecord) -> Self {
        Self {
            contributions: record.contributions.clone(),
            record,
        }
    }
}
