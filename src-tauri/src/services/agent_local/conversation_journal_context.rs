use super::ConversationJournal;
use crate::services::agent_local::context_usage_record::{
    ContextMeasurementSnapshot, ContextOutputSnapshot, ContextPreparationSnapshot,
    ContextPreparationState, ContextRequestIdentity, ContextUsageRecord,
};

impl ConversationJournal {
    pub(crate) fn context_identity(
        &self,
        turn: u32,
        attempt: u32,
        provider_id: &str,
        model: &str,
    ) -> ContextRequestIdentity {
        ContextRequestIdentity {
            request_id: self.request_id.clone(),
            turn_id: self.turn_id.clone(),
            turn,
            attempt,
            provider_id: provider_id.to_string(),
            model: model.to_string(),
        }
    }

    pub(crate) async fn activate_context_request(&self) -> Result<(), String> {
        let request_id = self.request_id.clone();
        self.update(move |session| {
            if let Some(current) = &mut session.context_usage.current_preparation {
                current.state = ContextPreparationState::Stale;
                current.updated_at = chrono::Utc::now();
            }
            session.context_usage.active_request_id = Some(request_id);
            session.updated_at = Some(chrono::Utc::now());
            Ok(())
        })
        .await
    }

    pub(crate) async fn persist_context_preparation(
        &self,
        preparation: ContextPreparationSnapshot,
    ) -> Result<bool, String> {
        validate_preparation(&preparation)?;
        if !self.matches_identity(&preparation.identity) {
            return Err(super::error());
        }
        let request_id = self.request_id.clone();
        self.update_if_active(move |record| {
            if is_older_than_current(record, &preparation.identity) {
                return false;
            }
            record.current_preparation = Some(preparation);
            true
        }, &request_id)
        .await
    }

    pub(crate) async fn persist_context_measurement(
        &self,
        measurement: ContextMeasurementSnapshot,
    ) -> Result<bool, String> {
        validate_measurement(&measurement)?;
        if !self.matches_identity(&measurement.identity) {
            return Err(super::error());
        }
        let request_id = self.request_id.clone();
        self.update_if_active(move |record| {
            if !matches_current(record, &measurement.identity) {
                return false;
            }
            record.last_measurement = Some(measurement);
            true
        }, &request_id)
        .await
    }

    pub(crate) async fn persist_context_output(
        &self,
        output: ContextOutputSnapshot,
    ) -> Result<bool, String> {
        validate_output(&output)?;
        if !self.matches_identity(&output.identity) {
            return Err(super::error());
        }
        let request_id = self.request_id.clone();
        self.update_if_active(move |record| {
            if !matches_current(record, &output.identity) {
                return false;
            }
            record.last_output = Some(output);
            true
        }, &request_id)
        .await
    }

    pub(crate) async fn finish_context_request(
        &self,
        state: ContextPreparationState,
    ) -> Result<(), String> {
        if !matches!(
            state,
            ContextPreparationState::Completed
                | ContextPreparationState::Interrupted
                | ContextPreparationState::Failed
        ) {
            return Err(super::error());
        }
        let request_id = self.request_id.clone();
        self.update(move |session| {
            if session.context_usage.active_request_id.as_deref() != Some(&request_id) {
                return Ok(());
            }
            if let Some(current) = &mut session.context_usage.current_preparation {
                if current.identity.request_id == request_id {
                    current.state = state;
                    current.updated_at = chrono::Utc::now();
                }
            }
            session.context_usage.active_request_id = None;
            session.updated_at = Some(chrono::Utc::now());
            Ok(())
        })
        .await
    }

    pub(crate) async fn complete_context_attempt(
        &self,
        identity: &ContextRequestIdentity,
    ) -> Result<bool, String> {
        if !self.matches_identity(identity) {
            return Err(super::error());
        }
        let identity = identity.clone();
        let request_id = self.request_id.clone();
        self.update_if_active(
            move |record| {
                let Some(current) = &mut record.current_preparation else {
                    return false;
                };
                if current.identity != identity {
                    return false;
                }
                current.state = ContextPreparationState::Completed;
                current.updated_at = chrono::Utc::now();
                true
            },
            &request_id,
        )
        .await
    }

    pub(crate) async fn context_record(&self) -> Result<ContextUsageRecord, String> {
        super::super::session_store::get(&self.session_id)
            .await
            .map(|session| session.context_usage)
            .map_err(|_| super::error())
    }

    fn matches_identity(&self, identity: &ContextRequestIdentity) -> bool {
        identity.request_id == self.request_id && identity.turn_id == self.turn_id
    }

    async fn update_if_active<F>(&self, update: F, request_id: &str) -> Result<bool, String>
    where
        F: FnOnce(&mut ContextUsageRecord) -> bool,
    {
        let mut changed = false;
        self.update(|session| {
            if session.context_usage.active_request_id.as_deref() == Some(request_id) {
                changed = update(&mut session.context_usage);
                if changed {
                    session.updated_at = Some(chrono::Utc::now());
                }
            }
            Ok(())
        })
        .await?;
        Ok(changed)
    }
}

fn is_older_than_current(record: &ContextUsageRecord, identity: &ContextRequestIdentity) -> bool {
    record.current_preparation.as_ref().is_some_and(|current| {
        current.identity.request_id == identity.request_id
            && (identity.turn, identity.attempt)
                < (current.identity.turn, current.identity.attempt)
    })
}

fn matches_current(record: &ContextUsageRecord, identity: &ContextRequestIdentity) -> bool {
    record
        .current_preparation
        .as_ref()
        .is_some_and(|current| current.identity == *identity)
}

fn validate_preparation(value: &ContextPreparationSnapshot) -> Result<(), String> {
    ContextUsageRecord {
        current_preparation: Some(value.clone()),
        ..ContextUsageRecord::default()
    }
    .validate()
    .map_err(|_| super::error())
}

fn validate_measurement(value: &ContextMeasurementSnapshot) -> Result<(), String> {
    ContextUsageRecord {
        last_measurement: Some(value.clone()),
        ..ContextUsageRecord::default()
    }
    .validate()
    .map_err(|_| super::error())
}

fn validate_output(value: &ContextOutputSnapshot) -> Result<(), String> {
    ContextUsageRecord {
        last_output: Some(value.clone()),
        ..ContextUsageRecord::default()
    }
    .validate()
    .map_err(|_| super::error())
}
