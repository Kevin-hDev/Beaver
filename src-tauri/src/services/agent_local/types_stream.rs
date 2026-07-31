use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenPhase {
    Work,
    Final,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum StreamEvent {
    #[serde(rename_all = "camelCase")]
    Token {
        content: String,
        token_count: u32,
        tps: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        phase: Option<TokenPhase>,
    },
    ContentPhase {
        phase: TokenPhase,
    },
    Thinking {
        content: String,
        token_count: u32,
    },
    #[serde(rename_all = "camelCase")]
    ContextUsage {
        input_tokens: u32,
        output_tokens: u32,
        context_limit: u32,
        estimated: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        breakdown: Option<super::context_usage_buckets::RequestContextUsage>,
    },
    GenerationStarted {},
    ToolCall {
        name: String,
        arguments: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        domain: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    ToolResult {
        name: String,
        content: String,
        is_error: bool,
        #[serde(default)]
        truncated: bool,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        display_summary: Option<String>,
        tool_call_index: usize,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        resolved_path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        domain: Option<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        affected_paths: Vec<String>,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        file_changes: Vec<super::types_tools::ToolFileChange>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        start_line: Option<usize>,
    },
    TurnEnd {},
    #[serde(rename_all = "camelCase")]
    PermissionRequest {
        id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    #[serde(rename_all = "camelCase")]
    Done {
        eval_count: Option<u32>,
        eval_duration_ns: u64,
        final_tps: f64,
        tps_estimated: bool,
        prompt_tokens: Option<u32>,
        context_tokens: Option<u32>,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        message: String,
        #[serde(default)]
        is_connection: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        diagnostic:
            Option<crate::services::agent_local::types_diagnostics::AgentErrorDiagnosticSummary>,
    },
    #[serde(rename_all = "camelCase")]
    Notice {
        message_key: String,
    },
    #[serde(rename_all = "camelCase")]
    RetryIndicator {
        reason_key: String,
        attempt: u32,
        max_attempts: u32,
    },
    Compressing {
        status: String,
    },
    CompressionComplete {},
    #[serde(rename_all = "camelCase")]
    SessionSnapshot {
        messages: Vec<crate::services::agent_local::types_session::AgentMessage>,
        token_count: u32,
    },
    #[serde(rename_all = "camelCase")]
    SubagentSpawned {
        subagent_session_id: String,
        subagent_name: String,
        subagent_type: String,
        subagent_description: String,
        subagent_color_key: String,
        prompt_preview: String,
        run_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    SubagentCompleted {
        subagent_session_id: String,
        success: bool,
        status: String,
        summary: String,
        run_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    TodoUpdated {
        todos: Vec<crate::services::agent_local::types_todo::AgentTodoItem>,
    },
    #[serde(rename_all = "camelCase")]
    PlanPreviewUpdated {
        plan: Option<crate::services::agent_local::types_plan::AgentPlanPreview>,
    },
    #[serde(rename_all = "camelCase")]
    PlanModeUpdated {
        enabled: bool,
    },
    #[serde(rename_all = "camelCase")]
    InteractiveChoiceRequest {
        session_id: String,
        id: String,
        kind: crate::services::agent_local::types_interactive::AgentInteractiveChoiceKind,
        questions: Vec<crate::services::agent_local::types_interactive::AgentInteractiveQuestion>,
        current_index: usize,
        total: usize,
    },
}

#[derive(Debug, Default)]
pub struct StreamResult {
    pub content: String,
    pub content_chunks: Vec<String>,
    pub thinking: String,
    pub(crate) generated_units: usize,
    pub tool_calls: Vec<(String, serde_json::Value)>,
    /// IDs OpenAI-compat alignés avec `tool_calls` (vide pour Ollama).
    pub tool_call_ids: Vec<String>,
    pub tool_call_extra_content: Vec<Option<serde_json::Value>>,
    pub eval_count: Option<u32>,
    pub prompt_tokens: Option<u32>,
    pub usage: Option<crate::services::provider_usage::RequestUsage>,
    pub generation: super::generation_metrics::GenerationTracker,
    /// Diagnostic Ollama : raison de fin renvoyée par le chunk `done:true`
    /// (ex: "stop", "length", "tool_call"). None si le champ est absent.
    pub done_reason: Option<String>,
    /// Diagnostic Ollama : durée totale (ns) annoncée par le chunk final.
    pub total_duration_ns: Option<u64>,
    /// Diagnostic : nombre total de chunks NDJSON reçus d'Ollama.
    pub total_chunks: u32,
    /// Diagnostic : chunks reçus sans contenu/thinking/tool_call ni `done:true`.
    /// Piste du bug "stream s'arrête après un tool" (cf. pydantic-ai #1292).
    pub empty_chunks: u32,
}

#[derive(Debug)]
pub enum StreamOutcome {
    Completed(StreamResult),
    InterruptedForCompression(StreamResult),
}

impl StreamOutcome {
    pub fn into_result(self) -> StreamResult {
        match self {
            Self::Completed(result) | Self::InterruptedForCompression(result) => result,
        }
    }

    pub fn is_interrupted(&self) -> bool {
        matches!(self, Self::InterruptedForCompression(_))
    }
}

#[cfg(test)]
#[path = "types_stream_tests.rs"]
mod tests;
