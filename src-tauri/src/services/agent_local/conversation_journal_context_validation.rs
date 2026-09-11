use crate::services::agent_local::context_usage_record::{
    ContextMeasurementSnapshot, ContextOutputSnapshot, ContextPreparationSnapshot,
    ContextRequestIdentity, ContextUsageRecord,
};

pub(super) fn is_older_than_current(
    record: &ContextUsageRecord,
    identity: &ContextRequestIdentity,
) -> bool {
    record.current_preparation.as_ref().is_some_and(|current| {
        current.identity.request_id == identity.request_id
            && (identity.turn, identity.attempt)
                < (current.identity.turn, current.identity.attempt)
    })
}

pub(super) fn matches_current(
    record: &ContextUsageRecord,
    identity: &ContextRequestIdentity,
) -> bool {
    record
        .current_preparation
        .as_ref()
        .is_some_and(|current| current.identity == *identity)
}

pub(super) fn preparation(value: &ContextPreparationSnapshot) -> Result<(), String> {
    validate(ContextUsageRecord {
        current_preparation: Some(value.clone()),
        ..ContextUsageRecord::default()
    })
}

pub(super) fn measurement(value: &ContextMeasurementSnapshot) -> Result<(), String> {
    validate(ContextUsageRecord {
        last_measurement: Some(value.clone()),
        ..ContextUsageRecord::default()
    })
}

pub(super) fn output(value: &ContextOutputSnapshot) -> Result<(), String> {
    validate(ContextUsageRecord {
        last_output: Some(value.clone()),
        ..ContextUsageRecord::default()
    })
}

fn validate(record: ContextUsageRecord) -> Result<(), String> {
    record.validate().map_err(|_| super::super::error())
}
