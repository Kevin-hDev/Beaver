use chrono::Utc;

use super::context_usage_buckets::RequestContextUsage;
use super::context_usage_record::{
    ContextCountCoverage, ContextCountSource, ContextMeasurementSnapshot,
    ContextOutputSnapshot, ContextPreparationSnapshot, ContextPreparationState,
    ContextTokenCount,
};
use super::conversation_journal::ConversationJournal;
use super::stream_events::AgentEventEmitter;
use super::types_stream::{StreamEvent, StreamResult};
use crate::services::token_counting;

pub struct ContextAttempt<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub journal: Option<&'a ConversationJournal>,
    pub provider_id: &'a str,
    pub model: &'a str,
    pub turn: usize,
    pub attempt: u32,
    pub context_limit: u64,
    pub measured_input_source: ContextCountSource,
}

impl ContextAttempt<'_> {
    pub async fn persist_preparation(
        &self,
        input_tokens: usize,
        breakdown: RequestContextUsage,
    ) -> Result<u32, String> {
        let input_tokens = bounded_tokens(input_tokens);
        let Some(journal) = self.journal else {
            return Ok(input_tokens);
        };
        let preparation = ContextPreparationSnapshot {
            identity: journal.context_identity(
                bounded_tokens(self.turn),
                self.attempt,
                self.provider_id,
                self.model,
            ),
            context_limit: bounded_limit(self.context_limit),
            input: complete_count(input_tokens, ContextCountSource::Heuristic),
            state: ContextPreparationState::InFlight,
            breakdown: Some(breakdown),
            updated_at: Utc::now(),
        };
        if journal.persist_context_preparation(preparation).await? {
            emit_record(self.on_event, journal).await?;
        }
        Ok(input_tokens)
    }

    pub async fn persist_result(&self, result: &StreamResult) -> Result<(), String> {
        let Some(journal) = self.journal else {
            return Ok(());
        };
        let identity = journal.context_identity(
            bounded_tokens(self.turn),
            self.attempt,
            self.provider_id,
            self.model,
        );
        let (input, output) = resolved_result_counts(result, self.measured_input_source);
        if let Some((tokens, source)) = input {
            let context_limit = journal
                .context_record()
                .await?
                .current_preparation
                .as_ref()
                .filter(|value| value.identity == identity)
                .and_then(|value| value.context_limit);
            journal
                .persist_context_measurement(ContextMeasurementSnapshot {
                    identity: identity.clone(),
                    context_limit,
                    input: complete_count(tokens, source),
                    updated_at: Utc::now(),
                })
                .await?;
        }
        if let Some((tokens, source)) = output {
            journal
                .persist_context_output(ContextOutputSnapshot {
                    identity: identity.clone(),
                    output: complete_count(tokens, source),
                    updated_at: Utc::now(),
                })
                .await?;
        }
        journal.complete_context_attempt(&identity).await?;
        emit_record(self.on_event, journal).await
    }
}

impl StreamResult {
    pub fn record_generated_text(&mut self, text: &str) -> u32 {
        self.generated_units = self
            .generated_units
            .saturating_add(token_counting::text_units(text));
        self.estimated_output_tokens()
    }

    pub fn record_generated_tool_call(&mut self, name: &str, arguments: &serde_json::Value) {
        self.generated_units = self
            .generated_units
            .saturating_add(token_counting::text_units(name))
            .saturating_add(token_counting::text_units(&arguments.to_string()));
    }

    pub fn estimated_output_tokens(&self) -> u32 {
        bounded_tokens(token_counting::token_count_from_units(self.generated_units))
    }
}

type ResolvedCount = Option<(u32, ContextCountSource)>;

fn resolved_result_counts(
    result: &StreamResult,
    measured_input_source: ContextCountSource,
) -> (ResolvedCount, ResolvedCount) {
    let input = result
        .prompt_tokens
        .map(|tokens| (tokens, measured_input_source));
    let provider_output = result
        .usage
        .as_ref()
        .and_then(|usage| usage.output_tokens);
    let output = if let Some(tokens) = provider_output {
        tokens
            .try_into()
            .ok()
            .map(|tokens| (tokens, ContextCountSource::Provider))
    } else {
        result
            .eval_count
            .map(|tokens| (tokens, ContextCountSource::NativeCounter))
            .or_else(|| {
                (result.generated_units > 0).then(|| {
                    (
                        result.estimated_output_tokens(),
                        ContextCountSource::Heuristic,
                    )
                })
            })
    };
    (input, output)
}

fn complete_count(tokens: u32, source: ContextCountSource) -> ContextTokenCount {
    ContextTokenCount {
        tokens: Some(tokens),
        capacity_tokens: Some(tokens),
        source: Some(source),
        coverage: ContextCountCoverage::Complete,
    }
}

async fn emit_record(
    on_event: &AgentEventEmitter,
    journal: &ConversationJournal,
) -> Result<(), String> {
    let record = journal.context_record().await?;
    let _ = on_event.send(StreamEvent::ContextUsage { record });
    Ok(())
}

fn bounded_limit(limit: u64) -> Option<u32> {
    (limit > 0).then(|| limit.min(u32::MAX as u64) as u32)
}

fn bounded_tokens(tokens: usize) -> u32 {
    tokens.min(u32::MAX as usize) as u32
}

#[cfg(test)]
#[path = "context_usage_runtime_tests.rs"]
mod tests;
