use super::memory_paths::MemoryLayout;
use super::memory_runtime::{self, MemoryTurnGuard};
use super::memory_types::{MemoryMode, MemorySettings};
use super::types_ollama::ChatMessage;
use std::path::Path;

const SUMMARY_CONTEXT_MAX: usize = 800;
const UNKNOWN_WINDOW_BUDGET: usize = 512;

pub struct PreparedMemory {
    pub section: Option<String>,
    pub tokens: usize,
    pub guard: MemoryTurnGuard,
}

pub async fn prepare(
    session_id: &str,
    messages: &[ChatMessage],
    working_dir: &Path,
    context_window: u64,
    is_subagent: bool,
) -> PreparedMemory {
    let settings = super::memory_settings::load().await;
    let explicit = latest_user_message(messages)
        .is_some_and(memory_runtime::has_explicit_request);
    if !settings.mode.is_active() {
        return inactive(session_id);
    }
    if is_subagent {
        return prepare_subagent(session_id, &settings, working_dir, context_window).await;
    }
    let (section, usage, total_budget) =
        build_section(&settings, session_id, working_dir, context_window, explicit).await;
    let tokens = usage.total();
    let guard = memory_runtime::begin(
        session_id,
        settings.mode,
        explicit,
        total_budget,
        tokens,
    );
    PreparedMemory {
        section: (!section.is_empty()).then_some(section),
        tokens,
        guard,
    }
}

async fn prepare_subagent(
    session_id: &str,
    settings: &MemorySettings,
    working_dir: &Path,
    context_window: u64,
) -> PreparedMemory {
    let total_budget = memory_budget(context_window, settings.context_budget_tokens);
    let layout = MemoryLayout::production();
    let project = layout.project_scope_ready(working_dir).await.ok();
    let section = super::memory_prompt::subagent_section(
        &layout.global_scope(),
        project.as_ref(),
    );
    let section = memory_runtime::truncate_to_tokens(&section, total_budget.min(256));
    let tokens = estimate(&section);
    let guard = memory_runtime::begin(
        session_id,
        MemoryMode::Manual,
        false,
        total_budget,
        tokens,
    );
    PreparedMemory {
        section: Some(section),
        tokens,
        guard,
    }
}

pub async fn estimate_usage(
    working_dir: &Path,
    context_window: u64,
) -> super::memory_context_usage::MemoryContextUsage {
    let settings = super::memory_settings::load().await;
    if !settings.mode.is_active() {
        return Default::default();
    }
    let (_, usage, _) =
        build_section(&settings, "00000000-0000-4000-8000-000000000000", working_dir, context_window, false)
            .await;
    usage
}

async fn build_section(
    settings: &MemorySettings,
    session_id: &str,
    working_dir: &Path,
    context_window: u64,
    explicit: bool,
) -> (
    String,
    super::memory_context_usage::MemoryContextUsage,
    usize,
) {
    let total_budget = memory_budget(context_window, settings.context_budget_tokens);
    let layout = MemoryLayout::production();
    let global = layout.global_scope();
    let project = layout.project_scope_ready(working_dir).await.ok();
    let global_summary = super::memory_store::load_summary(&global).await;
    let project_summary = match project.as_ref() {
        Some(scope) => super::memory_store::load_summary(scope).await,
        None => String::new(),
    };
    let rules = super::memory_prompt::main_section(
        settings.mode,
        explicit,
        session_id,
        &global,
        project.as_ref(),
    );
    let rules_tokens = estimate(&rules);
    let summary_budget = SUMMARY_CONTEXT_MAX
        .min(total_budget)
        .saturating_sub(rules_tokens);
    let summaries = memory_runtime::truncate_to_tokens(
        &super::memory_prompt::format_summaries(&global_summary, &project_summary),
        summary_budget,
    );
    let section = format!("{rules}{summaries}</memory_context>");
    let section = memory_runtime::truncate_to_tokens(
        &section,
        SUMMARY_CONTEXT_MAX.min(total_budget),
    );
    let usage = super::memory_context_usage::MemoryContextUsage::from_section(&section);
    (section, usage, total_budget)
}

fn latest_user_message(messages: &[ChatMessage]) -> Option<&str> {
    messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| message.content.as_str())
}

pub fn memory_budget(context_window: u64, configured_max: u32) -> usize {
    let configured = configured_max.min(3_000) as usize;
    if context_window == 0 {
        return configured.min(UNKNOWN_WINDOW_BUDGET);
    }
    let scaled = (context_window as usize)
        .saturating_mul(3_000)
        .checked_div(128_000)
        .unwrap_or(0)
        .clamp(512, 3_000);
    configured.min(scaled)
}

fn inactive(session_id: &str) -> PreparedMemory {
    PreparedMemory {
        section: None,
        tokens: 0,
        guard: memory_runtime::begin(session_id, MemoryMode::Disabled, false, 0, 0),
    }
}

fn estimate(content: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_is_capped_at_three_thousand_for_128k() {
        assert_eq!(memory_budget(128_000, 3_000), 3_000);
        assert_eq!(memory_budget(1_000_000, 3_000), 3_000);
        assert_eq!(memory_budget(64_000, 3_000), 1_500);
    }

    #[test]
    fn unknown_window_only_allows_a_compact_summary() {
        assert_eq!(memory_budget(0, 3_000), UNKNOWN_WINDOW_BUDGET);
    }
}
