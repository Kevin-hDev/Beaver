use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiCatalogEntry {
    pub extension_id: String,
    pub contribution_id: String,
    #[serde(skip)]
    pub action_ids: Vec<String>,
    #[serde(skip)]
    pub declared_action_ids: Vec<String>,
    pub contribution: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiCatalogSnapshot {
    pub revision: u64,
    pub contributions: Vec<UiCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BoundedFields(BTreeMap<String, Value>);

impl BoundedFields {
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn validate(&self) -> Result<(), String> {
        for (key, value) in &self.0 {
            super::validation::identifier(key)?;
            match value {
                Value::Null | Value::Bool(_) | Value::Number(_) => {}
                Value::String(text)
                    if text.chars().count() <= super::ui_contract::MAX_TEXT_CHARS => {}
                _ => return Err("invalid UI field".to_string()),
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for BoundedFields {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldsVisitor;
        impl<'de> serde::de::Visitor<'de> for FieldsVisitor {
            type Value = BoundedFields;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a bounded UI fields object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                if map
                    .size_hint()
                    .is_some_and(|size| size > super::ui_contract::MAX_FIELDS_PER_VIEW)
                {
                    return Err(serde::de::Error::custom("too many UI fields"));
                }
                let mut fields = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, Value>()? {
                    if fields.len() >= super::ui_contract::MAX_FIELDS_PER_VIEW {
                        return Err(serde::de::Error::custom("too many UI fields"));
                    }
                    if fields.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("duplicate UI field"));
                    }
                }
                Ok(BoundedFields(fields))
            }
        }
        deserializer.deserialize_map(FieldsVisitor)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiActionPayload {
    pub fields: BoundedFields,
}

impl UiActionPayload {
    pub(super) fn validate(&self) -> Result<(), String> {
        self.fields.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| "invalid UI payload".to_string())?;
        (bytes.len() <= super::ui_contract::MAX_ACTION_PAYLOAD_BYTES)
            .then_some(())
            .ok_or_else(|| "invalid UI payload".to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiActionRequest {
    pub extension_id: String,
    pub contribution_id: String,
    pub action_id: String,
    pub payload: UiActionPayload,
    pub locale: String,
}
