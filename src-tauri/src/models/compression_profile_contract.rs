use serde::{Deserialize, Serialize};

use crate::services::compress::profile_store_document::CompressionProfileDocument;
use crate::services::compress::profile_types::{
    CompressionBandSettings, CompressionCategory, CompressionProfile, CompressionSummarySettings,
    ContextCapacityPolicy,
};

pub type CompressionProfileView = CompressionProfile;
// Exported as a TypeScript alias by `typescript_bindings`.
#[allow(dead_code)]
pub type CompressionBandView = CompressionBandSettings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct CompressionProfileInput {
    pub id: String,
    pub name: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub revision: u64,
    pub threshold_percent: u8,
    pub allow_under_64k: bool,
    pub context_capacity_policy: ContextCapacityPolicy,
    pub summary: CompressionSummarySettings,
    pub under_64k: CompressionBandSettings,
    pub compact: CompressionBandSettings,
    pub large: CompressionBandSettings,
    pub reduction_order: Vec<CompressionCategory>,
}

impl From<CompressionProfileInput> for CompressionProfile {
    fn from(input: CompressionProfileInput) -> Self {
        Self {
            id: input.id,
            name: input.name,
            revision: input.revision,
            threshold_percent: input.threshold_percent,
            allow_under_64k: input.allow_under_64k,
            context_capacity_policy: input.context_capacity_policy,
            summary: input.summary,
            under_64k: input.under_64k,
            compact: input.compact,
            large: input.large,
            reduction_order: input.reduction_order,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CompressionProfilesView {
    pub global_profile_id: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub global_selection_revision: u64,
    pub profiles: Vec<CompressionProfileView>,
}

impl From<&CompressionProfileDocument> for CompressionProfilesView {
    fn from(document: &CompressionProfileDocument) -> Self {
        Self {
            global_profile_id: document.global_profile_id.clone(),
            global_selection_revision: document.global_selection_revision,
            profiles: document.profiles.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BudgetProjectionView {
    #[cfg_attr(test, ts(type = "number"))]
    pub context_window: u64,
    pub band: crate::services::compress::profile_types::CompressionWindowBand,
    pub system_tools_tokens: u32,
    pub summary_tokens: u32,
    pub categories_tokens: u32,
    pub reserve_tokens: u32,
    #[cfg_attr(test, ts(type = "number"))]
    pub total_tokens: u64,
    pub projected_percent: u8,
    pub exceeds_window: bool,
    pub high_risk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CompressionDeleteResult {
    pub view: CompressionProfilesView,
    pub undo_token: String,
    pub undo_expires_in_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
// Task 5 exposes this source through the effective per-session projection.
#[allow(dead_code)]
pub enum ResolvedCompressionProfileSource {
    Global,
    Session,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
// Task 5 constructs this view once session profile resolution is available.
#[allow(dead_code)]
pub struct ResolvedCompressionProfileView {
    pub id: String,
    pub name: String,
    pub source: ResolvedCompressionProfileSource,
    #[cfg_attr(test, ts(type = "number"))]
    pub profile_revision: u64,
    #[cfg_attr(test, ts(type = "number"))]
    pub global_selection_revision: u64,
    #[cfg_attr(test, ts(type = "number"))]
    pub context_window: u64,
    pub band: Option<crate::services::compress::profile_types::CompressionWindowBand>,
    pub available: bool,
}

#[cfg(test)]
pub(crate) fn typescript_bindings() -> String {
    use crate::services::compress::profile_types::*;
    use ts_rs::{Config, TS};

    let config = Config::default();
    let declarations = [
        CompressionWindowBand::decl(&config),
        BudgetMode::decl(&config),
        CompressionCategory::decl(&config),
        SummaryFailurePolicy::decl(&config),
        ContextCapacityPolicy::decl(&config),
        TokenBudget::decl(&config),
        CategoryBudget::decl(&config),
        ItemBudget::decl(&config),
        ImageBudget::decl(&config),
        SummaryOutputBudget::decl(&config),
        SummaryModelSelection::decl(&config),
        CompressionSummarySettings::decl(&config),
        CompressionBandSettings::decl(&config),
        CompressionProfile::decl(&config),
        CompressionProfileInput::decl(&config),
        CompressionProfilesView::decl(&config),
        BudgetProjectionView::decl(&config),
        CompressionDeleteResult::decl(&config),
        ResolvedCompressionProfileSource::decl(&config),
        ResolvedCompressionProfileView::decl(&config),
    ];
    let output = format!(
        "// @generated from Rust by `npm run contracts:generate:compression`.\n\
         // Do not edit this file manually.\n\n\
         export {}\n\n\
         export type CompressionProfileView = CompressionProfile;\n\
         export type CompressionBandView = CompressionBandSettings;\n",
        declarations.join("\n\nexport ")
    );
    output
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
