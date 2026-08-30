use std::path::Path;

use super::checkpoint_document::CheckpointSection;
use super::checkpoint_transaction::CompressionError;
use super::profile_types::{CompressionBandSettings, CompressionCategory, ContextCapacityPolicy};
use super::snapshot::CompressionSnapshot;
use super::summary_contract::ValidatedSummary;
use crate::services::agent_local::types_ollama::ChatMessage;

pub async fn build(
    snapshot: &CompressionSnapshot,
    summary: Option<&ValidatedSummary>,
    runtime: &[ChatMessage],
    working_dir: &Path,
) -> Result<super::checkpoint_candidate::CompressionCandidate, CompressionError> {
    let sections = super::orchestrator_sections::collect(snapshot, runtime, working_dir).await?;
    match super::checkpoint_candidate::build(snapshot, summary, &sections).await {
        Err(CompressionError::CapacityExceeded)
            if snapshot.profile.profile.context_capacity_policy
                == ContextCapacityPolicy::ReduceOptionalCategories =>
        {
            let mut reduced = snapshot.clone();
            if !reduce_first_contributing_category(&mut reduced, &sections) {
                return Err(CompressionError::CapacityExceeded);
            }
            let sections =
                super::orchestrator_sections::collect(&reduced, runtime, working_dir).await?;
            super::checkpoint_candidate::build(&reduced, summary, &sections).await
        }
        result => result,
    }
}

fn reduce_first_contributing_category(
    snapshot: &mut CompressionSnapshot,
    sections: &[CheckpointSection],
) -> bool {
    let order = snapshot.profile.profile.reduction_order.clone();
    for category in order {
        if reduce_if_present(snapshot, sections, category) {
            return true;
        }
    }
    false
}

fn reduce_if_present(
    snapshot: &mut CompressionSnapshot,
    sections: &[CheckpointSection],
    category: CompressionCategory,
) -> bool {
    let present = category_is_present(snapshot, sections, category);
    if !present {
        return false;
    }
    if category == CompressionCategory::Images {
        snapshot.checkpoint_images.clear();
    }
    let band = active_band_mut(snapshot);
    match category {
        CompressionCategory::Images => disable_item_like(&mut band.images.enabled),
        CompressionCategory::Files => disable_item_like(&mut band.files.enabled),
        CompressionCategory::ModifiedFiles => disable_item_like(&mut band.modified_files.enabled),
        CompressionCategory::TextAttachments => {
            disable_item_like(&mut band.text_attachments.enabled)
        }
        CompressionCategory::Tools => disable_item_like(&mut band.tools.enabled),
        CompressionCategory::AssistantMessages => {
            disable_item_like(&mut band.assistant_messages.enabled)
        }
        CompressionCategory::UserMessages => disable_item_like(&mut band.user_messages.enabled),
        CompressionCategory::Git => disable_item_like(&mut band.git_tokens.enabled),
        CompressionCategory::PlanAndTasks => {
            disable_item_like(&mut band.plan_and_tasks_tokens.enabled)
        }
        CompressionCategory::Subagents => {
            disable_item_like(&mut band.subagent_detail_tokens.enabled)
        }
        CompressionCategory::UnresolvedState => {
            disable_item_like(&mut band.unresolved_state_tokens.enabled)
        }
        CompressionCategory::CriticalReferences => {
            disable_item_like(&mut band.critical_references.enabled)
        }
    }
}

fn category_is_present(
    snapshot: &CompressionSnapshot,
    sections: &[CheckpointSection],
    category: CompressionCategory,
) -> bool {
    let has_role = |role: &str| {
        snapshot
            .source_messages
            .iter()
            .any(|message| message.role == role)
    };
    match category {
        CompressionCategory::Images => !snapshot.checkpoint_images.is_empty(),
        CompressionCategory::Files | CompressionCategory::ModifiedFiles => sections
            .iter()
            .any(|section| section.name == "files" && section.content != "[]"),
        CompressionCategory::TextAttachments | CompressionCategory::UserMessages => {
            has_role("user")
        }
        CompressionCategory::Tools => snapshot.source_messages.iter().any(|message| {
            message
                .tool_calls
                .as_ref()
                .is_some_and(|calls| !calls.is_empty())
                || message.role == "tool"
        }),
        CompressionCategory::AssistantMessages => has_role("assistant"),
        CompressionCategory::Git
        | CompressionCategory::PlanAndTasks
        | CompressionCategory::Subagents
        | CompressionCategory::UnresolvedState
        | CompressionCategory::CriticalReferences => true,
    }
}

fn active_band_mut(snapshot: &mut CompressionSnapshot) -> &mut CompressionBandSettings {
    match snapshot.profile.band(snapshot.context_window) {
        Some(super::profile_types::CompressionWindowBand::Under64K) => {
            &mut snapshot.profile.profile.under_64k
        }
        Some(super::profile_types::CompressionWindowBand::Large) => {
            &mut snapshot.profile.profile.large
        }
        Some(super::profile_types::CompressionWindowBand::Compact) | None => {
            &mut snapshot.profile.profile.compact
        }
    }
}

fn disable_item_like(enabled: &mut bool) -> bool {
    let changed = *enabled;
    *enabled = false;
    changed
}
