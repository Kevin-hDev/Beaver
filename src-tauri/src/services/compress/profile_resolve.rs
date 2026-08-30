use serde::{Deserialize, Serialize};

use super::profile_defaults::BEAVER_PROFILE_ID;
use super::profile_store::CompressionProfileStoreError;
use super::profile_store_document::CompressionProfileDocument;
use super::profile_types::{CompressionProfile, CompressionWindowBand};
use crate::services::agent_local::types_session::{
    AgentSession, SessionCompressionProfileSelection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum ResolvedCompressionProfileSource {
    Global,
    Session,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCompressionProfile {
    pub profile: CompressionProfile,
    pub profile_revision: u64,
    pub source: ResolvedCompressionProfileSource,
    pub global_selection_revision: u64,
}

impl ResolvedCompressionProfile {
    pub fn selection(&self) -> SessionCompressionProfileSelection {
        SessionCompressionProfileSelection {
            profile_id: self.profile.id.clone(),
            global_selection_revision: self.global_selection_revision,
        }
    }

    pub fn band(&self, context_window: u64) -> Option<CompressionWindowBand> {
        super::profile_budget::band_for_window(context_window)
    }

    pub fn available(&self, context_window: u64) -> bool {
        !matches!(
            self.band(context_window),
            Some(CompressionWindowBand::Under64K)
        ) || self.profile.allow_under_64k
    }
}

pub fn resolve_for_session(
    session: &AgentSession,
) -> Result<ResolvedCompressionProfile, CompressionProfileStoreError> {
    let document = super::profile_store::load_document()?;
    resolve_from_document(session.compression_profile_selection.as_ref(), &document)
}

pub(crate) fn resolve_from_document(
    selection: Option<&SessionCompressionProfileSelection>,
    document: &CompressionProfileDocument,
) -> Result<ResolvedCompressionProfile, CompressionProfileStoreError> {
    let selected = selection
        .filter(|choice| choice.global_selection_revision == document.global_selection_revision)
        .and_then(|choice| profile(document, &choice.profile_id))
        .map(|profile| (profile, ResolvedCompressionProfileSource::Session));
    let (profile, source) = selected
        .or_else(|| {
            profile(document, &document.global_profile_id)
                .map(|profile| (profile, ResolvedCompressionProfileSource::Global))
        })
        .or_else(|| {
            profile(document, BEAVER_PROFILE_ID)
                .map(|profile| (profile, ResolvedCompressionProfileSource::Global))
        })
        .ok_or(CompressionProfileStoreError::Invalid)?;
    Ok(ResolvedCompressionProfile {
        profile: profile.clone(),
        profile_revision: profile.revision,
        source,
        global_selection_revision: document.global_selection_revision,
    })
}

pub(crate) fn active_clone_selection(
    session: &AgentSession,
    document: &CompressionProfileDocument,
) -> Result<Option<SessionCompressionProfileSelection>, CompressionProfileStoreError> {
    active_selection(session.compression_profile_selection.as_ref(), document)
}

pub(crate) fn active_selection(
    selection: Option<&SessionCompressionProfileSelection>,
    document: &CompressionProfileDocument,
) -> Result<Option<SessionCompressionProfileSelection>, CompressionProfileStoreError> {
    let resolved = resolve_from_document(selection, document)?;
    Ok(
        (resolved.source == ResolvedCompressionProfileSource::Session)
            .then(|| resolved.selection()),
    )
}

fn profile<'a>(
    document: &'a CompressionProfileDocument,
    id: &str,
) -> Option<&'a CompressionProfile> {
    document.profiles.iter().find(|profile| profile.id == id)
}
