use serde::de::{Error, IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::RequestProjection;

#[path = "projection_scan.rs"]
mod scan;
#[path = "projection_tests.rs"]
mod tests;

const MAX_CAPTURE_ITEMS: usize = 2_048;
const MAX_MODEL_BYTES: usize = 128;
const MAX_TIER_BYTES: usize = 16;
const MAX_TYPE_BYTES: usize = 32;

#[derive(Deserialize)]
struct WireProjection {
    #[serde(deserialize_with = "model")]
    model: String,
    #[serde(default, deserialize_with = "tier")]
    service_tier: Option<String>,
    #[serde(rename = "type", default, deserialize_with = "envelope_type")]
    envelope_type: Option<String>,
    #[serde(default, deserialize_with = "count_items")]
    input: usize,
    #[serde(default, deserialize_with = "count_items")]
    tools: usize,
}

pub(super) fn parse(body_bytes: &[u8]) -> Result<RequestProjection, String> {
    if body_bytes.len() > crate::services::secure_http::LLM_BODY_LIMIT {
        return Err(invalid());
    }
    // Scan independently so ignored input/tool payloads cannot hide a sensitive field.
    let forbidden_field_present =
        scan::forbidden_field_present(body_bytes).map_err(|_| invalid())?;
    let wire: WireProjection = serde_json::from_slice(body_bytes).map_err(|_| invalid())?;
    Ok(RequestProjection {
        model: wire.model,
        service_tier: wire.service_tier,
        envelope_type: wire.envelope_type,
        input_count: wire.input,
        tool_count: wire.tools,
        forbidden_field_present,
        body_bytes: body_bytes.len(),
    })
}

fn model<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(SafeStringVisitor {
        max_bytes: MAX_MODEL_BYTES,
        model: true,
    })
}

fn tier<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    optional_string(deserializer, MAX_TIER_BYTES)
}

fn envelope_type<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    optional_string(deserializer, MAX_TYPE_BYTES)
}

fn optional_string<'de, D>(deserializer: D, max_bytes: usize) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionalVisitor {
        max_bytes: usize,
    }
    impl<'de> Visitor<'de> for OptionalVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded string or null")
        }

        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D: Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            deserializer
                .deserialize_str(SafeStringVisitor {
                    max_bytes: self.max_bytes,
                    model: false,
                })
                .map(Some)
        }
    }
    deserializer.deserialize_option(OptionalVisitor { max_bytes })
}

struct SafeStringVisitor {
    max_bytes: usize,
    model: bool,
}

impl Visitor<'_> for SafeStringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a validated bounded identifier")
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        let chars_valid = value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
        let model_valid =
            !self.model || crate::services::llm::runtime_models::valid_model_id(value);
        if value.is_empty() || value.len() > self.max_bytes || !chars_valid || !model_valid {
            return Err(E::custom("invalid identifier"));
        }
        Ok(value.to_string())
    }
}

fn count_items<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    struct CountVisitor;
    impl<'de> Visitor<'de> for CountVisitor {
        type Value = usize;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded array")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let mut count = 0;
            while sequence.next_element::<IgnoredAny>()?.is_some() {
                count += 1;
                if count > MAX_CAPTURE_ITEMS {
                    return Err(A::Error::custom("too many items"));
                }
            }
            Ok(count)
        }
    }
    deserializer.deserialize_seq(CountVisitor)
}

fn invalid() -> String {
    "provider_configuration_invalid".to_string()
}
