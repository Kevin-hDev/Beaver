use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCompressionProfileSelection {
    pub profile_id: String,
    pub global_selection_revision: u64,
}
