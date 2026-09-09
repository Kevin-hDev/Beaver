use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use serde::de::{Error as _, MapAccess, Visitor};
use serde::Deserializer;

use super::catalog_limits::MAX_LITELLM_CATALOG_ENTRIES;
use super::litellm_catalog::ModelEntry;

const TOO_MANY_MARKER: &str = "litellm_too_many_entries";
const DUPLICATE_MARKER: &str = "litellm_duplicate_id";
const INVALID_ENTRY_MARKER: &str = "litellm_invalid_entry";
const TECHNICAL_ENTRY_ID: &str = "sample_spec";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CatalogParseError {
    InvalidJson,
    TooManyEntries,
    DuplicateId,
    InvalidEntry,
}

impl CatalogParseError {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidJson => "invalid_json",
            Self::TooManyEntries => "too_many_entries",
            Self::DuplicateId => "duplicate_id",
            Self::InvalidEntry => "invalid_entry",
        }
    }
}

pub(crate) fn parse_catalog(json: &str) -> Result<HashMap<String, ModelEntry>, CatalogParseError> {
    parse_catalog_detailed(json).map_err(|(reason, _)| reason)
}

pub(crate) fn parse_catalog_detailed(
    json: &str,
) -> Result<HashMap<String, ModelEntry>, (CatalogParseError, usize)> {
    let input_count = Cell::new(0_usize);
    let mut deserializer = serde_json::Deserializer::from_str(json);
    let result = deserializer
        .deserialize_map(CatalogVisitor {
            input_count: &input_count,
        })
        .map_err(|error| (classify_error(error), input_count.get()))?;
    deserializer
        .end()
        .map_err(|_| (CatalogParseError::InvalidJson, input_count.get()))?;
    Ok(result)
}

struct CatalogVisitor<'a> {
    input_count: &'a Cell<usize>,
}

impl<'de> Visitor<'de> for CatalogVisitor<'_> {
    type Value = HashMap<String, ModelEntry>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a bounded LiteLLM model map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let capacity = map
            .size_hint()
            .unwrap_or_default()
            .min(MAX_LITELLM_CATALOG_ENTRIES);
        let mut result = HashMap::with_capacity(capacity);
        let mut seen = HashSet::with_capacity(capacity);
        while let Some(key) = map.next_key::<String>()? {
            let input_count = self.input_count.get().saturating_add(1);
            self.input_count.set(input_count);
            if input_count > MAX_LITELLM_CATALOG_ENTRIES {
                return Err(A::Error::custom(TOO_MANY_MARKER));
            }
            if !seen.insert(key.clone()) {
                return Err(A::Error::custom(DUPLICATE_MARKER));
            }
            let raw = map.next_value::<serde_json::Value>()?;
            if key == TECHNICAL_ENTRY_ID {
                continue;
            }
            let entry =
                serde_json::from_value(raw).map_err(|_| A::Error::custom(INVALID_ENTRY_MARKER))?;
            result.insert(key, entry);
        }
        Ok(result)
    }
}

fn classify_error(error: serde_json::Error) -> CatalogParseError {
    let message = error.to_string();
    if message.contains(TOO_MANY_MARKER) {
        CatalogParseError::TooManyEntries
    } else if message.contains(DUPLICATE_MARKER) {
        CatalogParseError::DuplicateId
    } else if message.contains(INVALID_ENTRY_MARKER) {
        CatalogParseError::InvalidEntry
    } else {
        CatalogParseError::InvalidJson
    }
}
