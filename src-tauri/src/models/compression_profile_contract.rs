use serde::{Deserialize, Serialize};

use crate::services::compress::profile_store_document::CompressionProfileDocument;
use crate::services::compress::profile_types::{CompressionBandSettings, CompressionProfile};

pub type CompressionProfileView = CompressionProfile;
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
    pub system_prompt: String,
    pub handoff_prompt: String,
    pub under_64k: CompressionBandSettings,
    pub compact: CompressionBandSettings,
    pub large: CompressionBandSettings,
}

impl From<CompressionProfileInput> for CompressionProfile {
    fn from(input: CompressionProfileInput) -> Self {
        Self {
            id: input.id,
            name: input.name,
            revision: input.revision,
            threshold_percent: input.threshold_percent,
            allow_under_64k: input.allow_under_64k,
            system_prompt: input.system_prompt,
            handoff_prompt: input.handoff_prompt,
            under_64k: input.under_64k,
            compact: input.compact,
            large: input.large,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CompressionProfilesView {
    pub automatic_enabled: bool,
    pub global_profile_id: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub global_selection_revision: u64,
    pub profiles: Vec<CompressionProfileView>,
}

impl From<&CompressionProfileDocument> for CompressionProfilesView {
    fn from(document: &CompressionProfileDocument) -> Self {
        Self {
            automatic_enabled: document.automatic_enabled,
            global_profile_id: document.global_profile_id.clone(),
            global_selection_revision: document.global_selection_revision,
            profiles: document.profiles.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BudgetProjectionView {
    pub band: crate::services::compress::profile_types::CompressionWindowBand,
    pub before_tokens: u32,
    pub system_tools_tokens: u32,
    pub variable_tokens: u32,
    pub target_tokens: u32,
    pub range_lower_tokens: u32,
    pub range_upper_tokens: u32,
    pub image_count: u16,
    pub projected_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CompressionDeleteResult {
    pub view: CompressionProfilesView,
    pub undo_token: String,
    pub undo_expires_in_ms: u32,
}

pub use crate::services::compress::profile_resolve::ResolvedCompressionProfileSource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
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

impl ResolvedCompressionProfileView {
    pub fn from_resolved(
        resolved: &crate::services::compress::profile_resolve::ResolvedCompressionProfile,
        context_window: u64,
    ) -> Self {
        Self {
            id: resolved.profile.id.clone(),
            name: resolved.profile.name.clone(),
            source: resolved.source,
            profile_revision: resolved.profile_revision,
            global_selection_revision: resolved.global_selection_revision,
            context_window,
            band: resolved.band(context_window),
            available: resolved.available(context_window),
        }
    }
}

#[cfg(test)]
pub(crate) fn typescript_bindings() -> String {
    use crate::services::compress::profile_types::*;
    use ts_rs::{Config, TS};

    let config = Config::default();
    let declarations = [
        CompressionWindowBand::decl(&config),
        CompressionTrigger::decl(&config),
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
