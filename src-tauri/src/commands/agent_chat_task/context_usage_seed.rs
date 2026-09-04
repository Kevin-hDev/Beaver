use super::common::PromptContext;
use crate::services::agent_local::context_usage_buckets::ContextUsageSeed;
use crate::services::agent_local::memory_context_usage::MemoryContextUsage;
use crate::services::git_context::GitSnapshot;

pub(super) fn for_prompt(context: &PromptContext<'_>) -> ContextUsageSeed {
    // Tool definitions, including the compact extension catalog, are measured separately in
    // RequestContextUsage::from_request as system_tools; adding them to this seed would double count.
    let mut meta = context
        .agent_md_content
        .as_deref()
        .map(|content| estimate(&format!("{content}\n\n")))
        .unwrap_or(0);
    meta = meta.saturating_add(git_tokens(context.snap));
    meta = meta.saturating_add(
        crate::services::agent_local::chat_prompt_sections::response_language_instruction(
            context.response_language,
        )
        .as_deref()
        .map(estimate)
        .unwrap_or(0),
    );
    if context.plan_mode_active {
        meta = meta.saturating_add(estimate(&format!(
            "\n\n{}",
            crate::services::agent_local::prompt_plan::plan_mode_prompt()
        )));
    }
    let skills =
        crate::services::agent_local::chat_prompt_sections::skills_listing_section(context.skills)
            .as_deref()
            .map(estimate)
            .unwrap_or(0);
    ContextUsageSeed {
        meta_context_tokens: meta,
        skill_context_tokens: skills,
        memory_context_tokens: memory_tokens(context.memory_context.as_deref()),
    }
}

pub(super) fn for_subagent(
    memory_context: Option<&str>,
    snapshot: &GitSnapshot,
) -> ContextUsageSeed {
    ContextUsageSeed {
        meta_context_tokens: git_tokens(snapshot),
        memory_context_tokens: memory_tokens(memory_context),
        ..Default::default()
    }
}

fn git_tokens(snapshot: &GitSnapshot) -> usize {
    crate::services::git_context::format_git_section(snapshot)
        .map(|section| estimate(&format!("\n\n{section}")))
        .unwrap_or(0)
}

fn memory_tokens(section: Option<&str>) -> usize {
    section
        .map(MemoryContextUsage::from_section)
        .map(|usage| usage.summary_tokens)
        .unwrap_or(0)
}

fn estimate(content: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(content)
}
