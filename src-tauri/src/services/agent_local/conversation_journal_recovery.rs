use chrono::Utc;

use super::ConversationJournal;
use crate::services::agent_local::stream_recovery_log::StreamRecoveryLog;
use crate::services::agent_local::stream_recovery_owners::OwnerLease;
use crate::services::agent_local::stream_recovery_record::{
    StreamRecoveryHeader, StreamRecoveryOwner, STREAM_RECOVERY_VERSION,
};

impl ConversationJournal {
    pub(crate) fn turn_ids(&self) -> (&str, &str, &str) {
        (
            &self.turn_id,
            &self.user_message_id,
            &self.assistant_message_id,
        )
    }

    pub(crate) fn recovery_header(&self) -> StreamRecoveryHeader {
        StreamRecoveryHeader {
            version: STREAM_RECOVERY_VERSION,
            process_instance_id: crate::services::agent_local::stream_recovery_record::process_instance_id().to_string(),
            session_id: self.session_id.clone(),
            request_id: self.request_id.clone(),
            turn_id: self.turn_id.clone(),
            user_message_id: self.user_message_id.clone(),
            assistant_message_id: self.assistant_message_id.clone(),
            subagent_owner: self.subagent_owner.as_ref().map(|owner| StreamRecoveryOwner {
                run_id: owner.run_id.clone(),
                execution_id: owner.execution_id.clone(),
            }),
            created_at: Utc::now(),
        }
    }

    pub(crate) fn attach_recovery(&mut self, log: StreamRecoveryLog, owner: OwnerLease) {
        self.recovery_log = Some(log);
        self.recovery_owner = Some(owner);
    }

    pub(super) fn verify_recovery(&self) -> Result<(), String> {
        self.recovery_log
            .as_ref()
            .and_then(StreamRecoveryLog::sticky_error)
            .map_or(Ok(()), Err)
    }

    pub(super) async fn append_staged(
        &self,
        records: Vec<crate::services::agent_local::types_message::AgentMessage>,
    ) -> Result<(), String> {
        self.verify_recovery()?;
        if let Some(log) = &self.recovery_log {
            log.stage_messages(records.clone()).await?;
        }
        self.append(records).await?;
        if let Some(log) = &self.recovery_log {
            log.clear_committed().await?;
        }
        Ok(())
    }
}
