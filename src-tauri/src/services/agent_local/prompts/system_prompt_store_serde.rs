use super::{PromptOverride, PromptPair};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
struct StoredPromptPair {
    compact: Option<StoredPromptOverride>,
    detailed: Option<StoredPromptOverride>,
    #[serde(default)]
    compact_beaver: bool,
    #[serde(default)]
    detailed_beaver: bool,
}

#[derive(Deserialize)]
#[serde(tag = "state", content = "content", rename_all = "lowercase")]
enum StoredPromptOverride {
    Custom(String),
    Disabled,
    Beaver,
}

impl<'de> Deserialize<'de> for PromptPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let stored = StoredPromptPair::deserialize(deserializer)?;
        let (compact, legacy_compact_beaver) = migrate(stored.compact);
        let (detailed, legacy_detailed_beaver) = migrate(stored.detailed);
        Ok(Self {
            compact,
            detailed,
            compact_beaver: stored.compact_beaver || legacy_compact_beaver,
            detailed_beaver: stored.detailed_beaver || legacy_detailed_beaver,
        })
    }
}

fn migrate(value: Option<StoredPromptOverride>) -> (Option<PromptOverride>, bool) {
    match value {
        Some(StoredPromptOverride::Custom(content)) => {
            (Some(PromptOverride::Custom(content)), false)
        }
        Some(StoredPromptOverride::Disabled) => (Some(PromptOverride::Disabled), false),
        Some(StoredPromptOverride::Beaver) => (None, true),
        None => (None, false),
    }
}
