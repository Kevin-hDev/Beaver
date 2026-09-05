use super::tool_artifact::EphemeralArtifact;

pub(crate) const MAX_OUTCOME_ARTIFACTS: usize =
    crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS
        * crate::services::extensions::types::MAX_RESULT_FILES;

#[derive(Debug)]
pub(crate) struct AttributedArtifact {
    pub tool_call_index: usize,
    pub tool_call_id: Option<String>,
    pub artifact: EphemeralArtifact,
}

#[derive(Default)]
pub(crate) struct ToolExecutionArtifacts(Vec<AttributedArtifact>);

impl ToolExecutionArtifacts {
    pub(crate) fn record(
        &mut self,
        tool_call_index: usize,
        tool_call_id: Option<&str>,
        artifacts: Vec<EphemeralArtifact>,
    ) -> Result<(), ()> {
        let count = self.0.len().checked_add(artifacts.len()).ok_or(())?;
        if count > MAX_OUTCOME_ARTIFACTS {
            return Err(());
        }
        self.0.extend(artifacts.into_iter().map(|artifact| AttributedArtifact {
            tool_call_index,
            tool_call_id: tool_call_id.map(str::to_owned),
            artifact,
        }));
        Ok(())
    }

    pub(crate) fn merge(&mut self, other: Self) -> Result<(), ()> {
        self.record_many(other.0)
    }

    pub(crate) fn as_slice(&self) -> &[AttributedArtifact] {
        &self.0
    }

    fn record_many(&mut self, artifacts: Vec<AttributedArtifact>) -> Result<(), ()> {
        let count = self.0.len().checked_add(artifacts.len()).ok_or(())?;
        if count > MAX_OUTCOME_ARTIFACTS {
            return Err(());
        }
        self.0.extend(artifacts);
        Ok(())
    }
}
