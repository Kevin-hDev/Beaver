use serde::{Deserialize, Serialize};

use super::types_tool_result::ToolResult;
use super::types_tools::ToolFileChange;

pub const MAX_AFFECTED_PATHS: usize = super::tool_file_changes::MAX_FILE_CHANGES;
pub(crate) const MAX_STORED_AFFECTED_PATHS: usize = 128;
pub(crate) const MAX_STORED_AFFECTED_PATH_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct ToolResultDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    display_summary: Option<Box<str>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    affected_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    file_changes: Vec<ToolFileChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    start_line: Option<usize>,
}

impl ToolResult {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn with_ephemeral_artifact(
        mut self,
        artifact: super::tool_artifact::EphemeralArtifact,
    ) -> Self {
        self.artifacts.ephemeral.push(artifact);
        self
    }

    pub(crate) fn push_ephemeral_artifact(
        &mut self,
        artifact: super::tool_artifact::EphemeralArtifact,
    ) {
        self.artifacts.ephemeral.push(artifact);
    }

    #[cfg(test)]
    pub(crate) fn ephemeral_artifacts(&self) -> &[super::tool_artifact::EphemeralArtifact] {
        &self.artifacts.ephemeral
    }

    pub(crate) fn take_ephemeral_artifacts(
        &mut self,
    ) -> Vec<super::tool_artifact::EphemeralArtifact> {
        std::mem::take(&mut self.artifacts.ephemeral)
    }

    pub(crate) fn set_pending_artifacts(
        &mut self,
        artifacts: Vec<super::tool_artifact::PendingArtifact>,
    ) -> Result<(), ()> {
        if self.is_error || artifacts.len() > crate::services::extensions::types::MAX_RESULT_FILES {
            return Err(());
        }
        self.artifacts.pending = artifacts;
        Ok(())
    }

    pub(crate) fn pending_artifacts(&self) -> &[super::tool_artifact::PendingArtifact] {
        &self.artifacts.pending
    }

    pub(crate) fn take_pending_artifacts(&mut self) -> Vec<super::tool_artifact::PendingArtifact> {
        std::mem::take(&mut self.artifacts.pending)
    }

    pub(crate) fn set_pending_extension_resource(
        &mut self,
        resource: super::tool_artifact::PendingExtensionResource,
    ) -> Result<(), ()> {
        if self.is_error || self.artifacts.pending_resource.is_some() {
            return Err(());
        }
        self.artifacts.pending_resource = Some(resource);
        Ok(())
    }

    pub(crate) fn take_pending_extension_resource(
        &mut self,
    ) -> Option<super::tool_artifact::PendingExtensionResource> {
        self.artifacts.pending_resource.take()
    }

    pub(crate) fn has_pending_extension_resource(&self) -> bool {
        self.artifacts.pending_resource.is_some()
    }

    pub(crate) fn pending_extension_resource(
        &self,
    ) -> Option<&super::tool_artifact::PendingExtensionResource> {
        self.artifacts.pending_resource.as_ref()
    }

    pub fn with_affected_paths(mut self, paths: Vec<String>) -> Self {
        self.details.affected_paths = paths;
        self
    }

    pub fn with_display_summary(mut self, summary: impl Into<String>) -> Self {
        self.details.display_summary = Some(summary.into().into_boxed_str());
        self
    }

    pub fn with_file_changes(mut self, changes: Vec<ToolFileChange>) -> Self {
        self.details.file_changes = changes;
        self
    }

    pub fn with_start_line(mut self, start_line: usize) -> Self {
        self.details.start_line = Some(start_line);
        self
    }

    pub fn display_summary(&self) -> Option<&str> {
        self.details.display_summary.as_deref()
    }

    pub fn affected_paths(&self) -> &[String] {
        &self.details.affected_paths
    }

    pub fn affected_paths_mut(&mut self) -> &mut Vec<String> {
        &mut self.details.affected_paths
    }

    pub fn file_changes(&self) -> &[ToolFileChange] {
        &self.details.file_changes
    }

    pub fn file_changes_mut(&mut self) -> &mut Vec<ToolFileChange> {
        &mut self.details.file_changes
    }

    pub fn bound_file_changes(&mut self) -> Option<(usize, usize)> {
        let total = self.details.file_changes.len();
        let changes = std::mem::take(&mut self.details.file_changes);
        let (sample, incomplete) = super::tool_file_changes::bounded_sample(changes);
        let stored = sample.len();
        self.details.file_changes = sample;
        incomplete.then_some((total, stored))
    }

    pub fn bound_affected_paths(&mut self) -> Option<(usize, usize)> {
        let paths = std::mem::take(&mut self.details.affected_paths);
        let total = paths.len();
        let (stored, incomplete) = bounded_affected_paths(paths);
        let stored_count = stored.len();
        self.details.affected_paths = stored;
        incomplete.then_some((total, stored_count))
    }

    pub fn start_line(&self) -> Option<usize> {
        self.details.start_line
    }
}

pub(crate) fn bounded_affected_paths(paths: Vec<String>) -> (Vec<String>, bool) {
    let total = paths.len();
    let mut stored = Vec::with_capacity(total.min(MAX_STORED_AFFECTED_PATHS));
    let mut serialized_bytes = 2_usize;
    for path in paths {
        if stored.len() >= MAX_STORED_AFFECTED_PATHS {
            break;
        }
        let Ok(encoded) = serde_json::to_vec(&path) else {
            break;
        };
        let separator = usize::from(!stored.is_empty());
        if serialized_bytes
            .saturating_add(separator)
            .saturating_add(encoded.len())
            > MAX_STORED_AFFECTED_PATH_BYTES
        {
            break;
        }
        serialized_bytes += separator + encoded.len();
        stored.push(path);
    }
    let incomplete = stored.len() < total;
    (stored, incomplete)
}
