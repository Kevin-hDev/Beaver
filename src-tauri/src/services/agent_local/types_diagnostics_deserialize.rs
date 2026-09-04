use serde::Deserialize;

use super::types_diagnostics::AgentExtensionDiagnostic;

#[derive(Deserialize)]
struct RawAgentExtensionDiagnostic {
    origin: String,
    reason: String,
    #[serde(default)]
    correlation_id: Option<String>,
    #[serde(
        default,
        alias = "related_search_ids",
        deserialize_with = "deserialize_inspection_ids"
    )]
    related_inspection_ids: Vec<String>,
    plugin_count: usize,
    plugin_ids: String,
    tool_count: usize,
    canonical_tool_names: String,
    provider_aliases: String,
    tool_delta: usize,
    #[serde(
        default,
        alias = "discovery_result_count",
        deserialize_with = "deserialize_inspection_result_count"
    )]
    inspection_result_count: usize,
    #[serde(
        default,
        alias = "discovery_result_plugin_ids",
        deserialize_with = "deserialize_inspection_result_plugin_ids"
    )]
    inspection_result_plugin_ids: String,
    provider_capacity_count: usize,
    provider_capacity_plugin_ids: String,
    global_capacity_count: usize,
    global_capacity_plugin_ids: String,
}

impl<'de> Deserialize<'de> for AgentExtensionDiagnostic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawAgentExtensionDiagnostic::deserialize(deserializer)?;
        Ok(Self {
            origin: raw.origin,
            reason: raw.reason,
            correlation_id: raw.correlation_id,
            related_inspection_ids: raw.related_inspection_ids,
            plugin_count: raw.plugin_count,
            plugin_ids: raw.plugin_ids,
            tool_count: raw.tool_count,
            canonical_tool_names: raw.canonical_tool_names,
            provider_aliases: raw.provider_aliases,
            tool_delta: raw.tool_delta,
            inspection_result_count: raw.inspection_result_count,
            inspection_result_plugin_ids: raw.inspection_result_plugin_ids,
            provider_capacity_count: raw.provider_capacity_count,
            provider_capacity_plugin_ids: raw.provider_capacity_plugin_ids,
            global_capacity_count: raw.global_capacity_count,
            global_capacity_plugin_ids: raw.global_capacity_plugin_ids,
        })
    }
}

fn deserialize_inspection_ids<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{SeqAccess, Visitor};

    struct InspectionIdsVisitor;

    impl<'de> Visitor<'de> for InspectionIdsVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded list of inspection identifiers")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut ids = Vec::with_capacity(super::tool_executor_parallel_batch::MAX_PARALLEL);
            while let Some(id) = sequence.next_element::<String>()? {
                if ids.len() < super::tool_executor_parallel_batch::MAX_PARALLEL
                    && uuid::Uuid::parse_str(&id).is_ok()
                    && !ids.contains(&id)
                {
                    ids.push(id);
                }
            }
            Ok(ids)
        }
    }

    deserializer.deserialize_seq(InspectionIdsVisitor)
}

fn deserialize_inspection_result_count<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(usize::deserialize(deserializer)?.min(crate::services::extensions::MAX_INSPECTED_EXTENSIONS))
}

fn deserialize_inspection_result_plugin_ids<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let mut bounded = String::new();
    for id in value
        .split(',')
        .filter(|id| crate::services::extensions::validate_identifier(id).is_ok())
        .take(crate::services::extensions::MAX_INSPECTED_EXTENSIONS)
    {
        let separator = usize::from(!bounded.is_empty());
        if bounded.chars().count() + separator + id.chars().count()
            > super::types_diagnostics::MAX_EXTENSION_DIAGNOSTIC_TEXT_CHARS
        {
            break;
        }
        if separator == 1 {
            bounded.push(',');
        }
        bounded.push_str(id);
    }
    Ok(bounded)
}

#[cfg(test)]
#[test]
fn legacy_search_ids_deserialize_as_bounded_inspection_ids() {
    let values = (0..12_u128)
        .map(|value| uuid::Uuid::from_u128(value + 1).to_string())
        .chain(["invalid".to_string()])
        .collect::<Vec<_>>();
    let diagnostic: AgentExtensionDiagnostic = serde_json::from_value(serde_json::json!({
        "origin": "extension_tools_refreshed",
        "reason": "selected",
        "related_search_ids": values,
        "plugin_count": 0,
        "plugin_ids": "",
        "tool_count": 0,
        "canonical_tool_names": "",
        "provider_aliases": "",
        "tool_delta": 0,
        "discovery_result_count": 999,
        "discovery_result_plugin_ids": "example.a,example.b,example.c,example.d,example.e",
        "provider_capacity_count": 0,
        "provider_capacity_plugin_ids": "",
        "global_capacity_count": 0,
        "global_capacity_plugin_ids": ""
    }))
    .unwrap();

    assert_eq!(
        diagnostic.related_inspection_ids.len(),
        super::tool_executor_parallel_batch::MAX_PARALLEL
    );
    assert!(diagnostic
        .related_inspection_ids
        .iter()
        .all(|id| uuid::Uuid::parse_str(id).is_ok()));
    assert_eq!(
        diagnostic.inspection_result_count,
        crate::services::extensions::MAX_INSPECTED_EXTENSIONS
    );
    assert_eq!(
        diagnostic.inspection_result_plugin_ids,
        "example.a,example.b,example.c,example.d"
    );
}

#[cfg(test)]
#[test]
fn new_diagnostic_serialization_uses_inspection_ids_only() {
    let id = uuid::Uuid::from_u128(1).to_string();
    let diagnostic = AgentExtensionDiagnostic {
        origin: "extension_tools_refreshed".to_string(),
        reason: "inspection_result".to_string(),
        correlation_id: None,
        related_inspection_ids: vec![id.clone()],
        plugin_count: 0,
        plugin_ids: String::new(),
        tool_count: 0,
        canonical_tool_names: String::new(),
        provider_aliases: String::new(),
        tool_delta: 0,
        inspection_result_count: 0,
        inspection_result_plugin_ids: String::new(),
        provider_capacity_count: 0,
        provider_capacity_plugin_ids: String::new(),
        global_capacity_count: 0,
        global_capacity_plugin_ids: String::new(),
    };

    let serialized = serde_json::to_value(diagnostic).unwrap();
    assert_eq!(serialized["related_inspection_ids"], serde_json::json!([id]));
    assert!(serialized.get("related_search_ids").is_none());
    assert!(serialized.get("discovery_result_count").is_none());
    assert!(serialized.get("discovery_result_plugin_ids").is_none());
    assert_eq!(serialized["inspection_result_count"], 0);
    assert_eq!(serialized["inspection_result_plugin_ids"], "");
}
