use serde::Deserialize;

use super::types_diagnostics::AgentExtensionDiagnostic;

#[derive(Deserialize)]
struct RawAgentExtensionDiagnostic {
    origin: String,
    reason: String,
    #[serde(default)]
    correlation_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_search_ids")]
    related_search_ids: Vec<String>,
    plugin_count: usize,
    plugin_ids: String,
    tool_count: usize,
    canonical_tool_names: String,
    provider_aliases: String,
    tool_delta: usize,
    discovery_result_count: usize,
    discovery_result_plugin_ids: String,
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
            related_search_ids: raw.related_search_ids,
            plugin_count: raw.plugin_count,
            plugin_ids: raw.plugin_ids,
            tool_count: raw.tool_count,
            canonical_tool_names: raw.canonical_tool_names,
            provider_aliases: raw.provider_aliases,
            tool_delta: raw.tool_delta,
            discovery_result_count: raw.discovery_result_count,
            discovery_result_plugin_ids: raw.discovery_result_plugin_ids,
            provider_capacity_count: raw.provider_capacity_count,
            provider_capacity_plugin_ids: raw.provider_capacity_plugin_ids,
            global_capacity_count: raw.global_capacity_count,
            global_capacity_plugin_ids: raw.global_capacity_plugin_ids,
        })
    }
}

fn deserialize_search_ids<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{SeqAccess, Visitor};

    struct SearchIdsVisitor;

    impl<'de> Visitor<'de> for SearchIdsVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded list of search identifiers")
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

    deserializer.deserialize_seq(SearchIdsVisitor)
}

#[cfg(test)]
#[test]
fn real_diagnostic_deserialization_bounds_and_validates_search_ids() {
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
        "discovery_result_count": 0,
        "discovery_result_plugin_ids": "",
        "provider_capacity_count": 0,
        "provider_capacity_plugin_ids": "",
        "global_capacity_count": 0,
        "global_capacity_plugin_ids": ""
    })).unwrap();

    assert_eq!(
        diagnostic.related_search_ids.len(),
        super::tool_executor_parallel_batch::MAX_PARALLEL
    );
    assert!(diagnostic
        .related_search_ids
        .iter()
        .all(|id| uuid::Uuid::parse_str(id).is_ok()));
}
