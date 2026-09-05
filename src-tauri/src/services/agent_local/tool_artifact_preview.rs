#[cfg(test)]
use std::sync::Arc;

use super::tool_artifact_replay::{
    replay, ArtifactReplay, ReplayedArtifact, UNSUPPORTED_PREVIEW_NOTE,
};

const MAX_NOTES: usize =
    crate::services::extensions::types::MAX_MULTIMODAL_PREVIEWS_PER_CONTINUATION;

/// Aperçus en mémoire du dernier lot d'outils : jamais sérialisés ni persistés.
#[derive(Debug)]
pub(crate) struct ToolResultPreview {
    pub tool_call_index: usize,
    pub tool_call_id: Option<String>,
    pub artifact: ReplayedArtifact,
}

#[derive(Debug)]
pub(crate) struct ToolResultPreviewNote {
    pub tool_call_index: usize,
    pub tool_call_id: Option<String>,
    pub text: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OmittedPreviewSource {
    pub tool_call_index: usize,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct ToolResultPreviewBatch {
    previews: Vec<ToolResultPreview>,
    omitted: usize,
    omitted_sources: Vec<OmittedPreviewSource>,
    notes: Vec<ToolResultPreviewNote>,
}

impl ToolResultPreviewBatch {
    pub(crate) async fn replay_from_artifacts(
        artifacts: Vec<super::tool_execution_artifacts::AttributedArtifact>,
    ) -> Self {
        let needs_workspace_key = artifacts.iter().any(|attributed| {
            attributed.artifact.metadata.purpose == super::tool_artifact::ArtifactPurpose::Preview
                && matches!(
                    attributed.artifact.metadata.source,
                    super::tool_artifact::ArtifactSource::WorkspaceFile { .. }
                )
        });
        let key = needs_workspace_key
            .then(crate::services::attachment_access::attachment_key)
            .transpose()
            .ok()
            .flatten();
        let mut batch = Self::default();
        for attributed in artifacts {
            let super::tool_execution_artifacts::AttributedArtifact {
                tool_call_index,
                tool_call_id,
                artifact,
            } = attributed;
            let super::tool_artifact::EphemeralArtifact { metadata, bytes } = artifact;
            if metadata.purpose != super::tool_artifact::ArtifactPurpose::Preview {
                continue;
            }
            let record = (&metadata).into();
            // P4 kept these bytes only until the durable reference was written.
            // Drop them before replay so one batch never occupies memory twice.
            drop(bytes);
            batch.push_replay(
                tool_call_index,
                tool_call_id,
                replay(&record, key.as_deref().map(Vec::as_slice)).await,
            );
        }
        batch
    }

    #[cfg(test)]
    pub(crate) fn from_ephemeral(
        tool_call_index: usize,
        tool_call_id: Option<String>,
        artifact: super::tool_artifact::EphemeralArtifact,
    ) -> Self {
        let purpose = match artifact.metadata.purpose {
            super::tool_artifact::ArtifactPurpose::Artifact => {
                super::tool_artifact_record::ToolArtifactPurpose::Artifact
            }
            super::tool_artifact::ArtifactPurpose::Preview => {
                super::tool_artifact_record::ToolArtifactPurpose::Preview
            }
        };
        let replay = ArtifactReplay {
            status: super::tool_artifact_record::ToolArtifactStatus::Intact,
            artifact: Some(ReplayedArtifact {
                name: artifact.metadata.name,
                mime_type: artifact.metadata.mime_type,
                purpose,
                bytes: Arc::from(artifact.bytes),
            }),
            note: None,
        };
        let mut batch = Self::default();
        batch.push_replay(tool_call_index, tool_call_id, replay);
        batch
    }

    #[cfg(test)]
    pub(crate) fn from_ephemerals(
        artifacts: impl IntoIterator<
            Item = (
                usize,
                Option<String>,
                super::tool_artifact::EphemeralArtifact,
            ),
        >,
    ) -> Self {
        let mut batch = Self::default();
        for (index, call_id, artifact) in artifacts {
            let purpose = match artifact.metadata.purpose {
                super::tool_artifact::ArtifactPurpose::Artifact => {
                    super::tool_artifact_record::ToolArtifactPurpose::Artifact
                }
                super::tool_artifact::ArtifactPurpose::Preview => {
                    super::tool_artifact_record::ToolArtifactPurpose::Preview
                }
            };
            batch.push_replay(
                index,
                call_id,
                ArtifactReplay {
                    status: super::tool_artifact_record::ToolArtifactStatus::Intact,
                    artifact: Some(ReplayedArtifact {
                        name: artifact.metadata.name,
                        mime_type: artifact.metadata.mime_type,
                        purpose,
                        bytes: Arc::from(artifact.bytes),
                    }),
                    note: None,
                },
            );
        }
        batch
    }

    pub(crate) fn previews(&self) -> &[ToolResultPreview] {
        &self.previews
    }

    pub(crate) fn omitted(&self) -> usize {
        self.omitted
    }

    pub(crate) fn omitted_sources(&self) -> &[OmittedPreviewSource] {
        &self.omitted_sources
    }

    pub(crate) fn notes(&self) -> &[ToolResultPreviewNote] {
        &self.notes
    }

    /// The next request has already consumed these bytes; conversation state
    /// keeps only the persisted textual artifact reference.
    pub(crate) fn clear_after_projection(&mut self) {
        self.previews.clear();
        self.omitted = 0;
        self.omitted_sources.clear();
        self.notes.clear();
    }

    fn push_replay(
        &mut self,
        tool_call_index: usize,
        tool_call_id: Option<String>,
        replay: ArtifactReplay,
    ) {
        let Some(mut artifact) = replay.artifact else {
            if let Some(text) = replay.note.filter(|_| self.notes.len() < MAX_NOTES) {
                self.notes.push(ToolResultPreviewNote {
                    tool_call_index,
                    tool_call_id,
                    text,
                });
            }
            return;
        };
        if artifact.purpose != super::tool_artifact_record::ToolArtifactPurpose::Preview {
            return;
        }
        let signature = crate::services::file_signature::classify(&artifact.bytes);
        if !signature.image() {
            if self.notes.len() < MAX_NOTES {
                self.notes.push(ToolResultPreviewNote {
                    tool_call_index,
                    tool_call_id,
                    text: UNSUPPORTED_PREVIEW_NOTE,
                });
            }
            return;
        }
        // The bytes, not persisted metadata, are the authority for provider MIME.
        artifact.mime_type = signature.mime().to_string();
        if self.previews.len() == MAX_NOTES {
            self.omitted = self.omitted.saturating_add(1);
            let source = OmittedPreviewSource {
                tool_call_index,
                tool_call_id,
            };
            if self.omitted_sources.len() < MAX_NOTES && !self.omitted_sources.contains(&source) {
                self.omitted_sources.push(source);
            }
            return;
        }
        self.previews.push(ToolResultPreview {
            tool_call_index,
            tool_call_id,
            artifact,
        });
    }
}

#[cfg(test)]
#[path = "tool_result_projection_sequence_tests.rs"]
mod sequence_tests;
