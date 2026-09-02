use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::tool_result_contract::{ToolErrorCategory, ToolResultStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AgentDiagnosticRun {
    pub request_id: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub generation: u64,
    pub status: String,
    pub severity: String,
    #[cfg_attr(test, ts(type = "string"))]
    pub started_at: DateTime<Utc>,
    #[cfg_attr(test, ts(type = "string"))]
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "string"))]
    pub ended_at: Option<DateTime<Utc>>,
    pub phase: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub last_tool: Option<AgentDiagnosticTool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(test, ts(optional, as = "Option<_>"))]
    pub recent_tools: Vec<AgentDiagnosticTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub active_todo: Option<AgentDiagnosticTodo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub safe_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(test, ts(optional, as = "Option<_>"))]
    pub events: Vec<AgentDiagnosticEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AgentDiagnosticTool {
    pub name: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional, type = "unknown"))]
    pub args: Option<serde_json::Value>,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub result_status: Option<ToolResultStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub error_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub error_category: Option<ToolErrorCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub retryable: Option<bool>,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default)]
    pub warning_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AgentDiagnosticTodo {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub active_task: Option<String>,
    pub completed: usize,
    pub total: usize,
    pub progress: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AgentDiagnosticEvent {
    #[cfg_attr(test, ts(type = "string"))]
    pub at: DateTime<Utc>,
    pub phase: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub extension: Option<AgentExtensionDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AgentExtensionDiagnostic {
    pub origin: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub correlation_id: Option<String>,
    #[serde(default)]
    #[cfg_attr(not(test), serde(deserialize_with = "deserialize_search_ids"))]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(test, ts(optional, as = "Option<_>"))]
    pub related_search_ids: Vec<String>,
    pub plugin_count: usize,
    pub plugin_ids: String,
    pub tool_count: usize,
    pub canonical_tool_names: String,
    pub provider_aliases: String,
    pub tool_delta: usize,
    pub discovery_result_count: usize,
    pub discovery_result_plugin_ids: String,
    pub provider_capacity_count: usize,
    pub provider_capacity_plugin_ids: String,
    pub global_capacity_count: usize,
    pub global_capacity_plugin_ids: String,
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
fn related_search_ids_are_bounded_and_validated_on_deserialization() {
    #[derive(Deserialize)]
    struct Fixture {
        #[serde(deserialize_with = "deserialize_search_ids")]
        ids: Vec<String>,
    }

    let values = (0..12_u128)
        .map(|value| uuid::Uuid::from_u128(value + 1).to_string())
        .chain(["invalid".to_string()])
        .collect::<Vec<_>>();
    let fixture: Fixture = serde_json::from_value(serde_json::json!({"ids": values})).unwrap();

    assert_eq!(
        fixture.ids.len(),
        super::tool_executor_parallel_batch::MAX_PARALLEL
    );
    assert!(fixture
        .ids
        .iter()
        .all(|id| uuid::Uuid::parse_str(id).is_ok()));
}

#[cfg(test)]
pub(crate) fn typescript_bindings() -> String {
    use ts_rs::{Config, TS};

    let config = Config::default();
    format!(
        "// @generated from Rust by `npm run contracts:generate:diagnostics`.\n\
         // Do not edit this file manually.\n\n\
         import type {{ ToolErrorCategory, ToolResultStatus }} from \"./agent-tool-result-contract\";\n\n\
         export {}\n\n\
         export {}\n\n\
         export {}\n\n\
         export {}\n\n\
         export {}\n",
        AgentDiagnosticTool::decl(&config),
        AgentDiagnosticTodo::decl(&config),
        AgentExtensionDiagnostic::decl(&config),
        AgentDiagnosticEvent::decl(&config),
        AgentDiagnosticRun::decl(&config),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentErrorDiagnosticSummary {
    pub request_id: String,
    pub phase: String,
    pub error_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
    pub safe_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStreamFailure {
    pub code: String,
    pub occurred_at: DateTime<Utc>,
    pub is_connection: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_todo_title: Option<String>,
}
