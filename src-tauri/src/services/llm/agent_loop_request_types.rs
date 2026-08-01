use crate::services::agent_local::generation_metrics::GenerationAggregate;
use crate::services::agent_local::types_ollama::StreamResult;

pub(super) struct ApiRequestOutput {
    pub result: StreamResult,
    pub plan_active: bool,
    pub interrupted: bool,
    pub input_tokens: u32,
    pub generation: GenerationAggregate,
}
