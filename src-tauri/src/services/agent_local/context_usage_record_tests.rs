use chrono::Utc;

use super::context_usage_record::{
    ContextCountCoverage, ContextCountSource, ContextMeasurementSnapshot,
    ContextOutputSnapshot, ContextPreparationSnapshot, ContextPreparationState,
    ContextRequestIdentity, ContextTokenCount, ContextUsageRecord,
};

fn identity() -> ContextRequestIdentity {
    ContextRequestIdentity {
        request_id: uuid::Uuid::new_v4().to_string(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        turn: 0,
        attempt: 1,
        provider_id: "openai".into(),
        model: "gpt-5".into(),
    }
}

fn count(tokens: u32, capacity_tokens: Option<u32>) -> ContextTokenCount {
    ContextTokenCount {
        tokens: Some(tokens),
        capacity_tokens,
        source: Some(ContextCountSource::Provider),
        coverage: ContextCountCoverage::Complete,
    }
}

fn measurement() -> ContextMeasurementSnapshot {
    ContextMeasurementSnapshot {
        identity: identity(),
        context_limit: Some(200_000),
        input: count(100, Some(100)),
        updated_at: Utc::now(),
    }
}

#[test]
fn valid_record_and_empty_record_are_accepted() {
    assert!(ContextUsageRecord::default().validate().is_ok());
    let measurement = measurement();
    let request_id = measurement.identity.request_id.clone();
    let record = ContextUsageRecord {
        active_request_id: Some(request_id),
        current_preparation: Some(ContextPreparationSnapshot {
            identity: identity(),
            context_limit: Some(200_000),
            input: count(120, Some(130)),
            state: ContextPreparationState::InFlight,
            breakdown: None,
            transient_overhead_tokens: 0,
            updated_at: Utc::now(),
        }),
        last_measurement: Some(measurement),
        last_output: Some(ContextOutputSnapshot {
            identity: identity(),
            output: count(50, None),
            updated_at: Utc::now(),
        }),
    };
    assert!(record.validate().is_ok());
}

#[test]
fn identities_are_uuid_backed_bounded_and_nonempty() {
    for mutate in [
        |value: &mut ContextRequestIdentity| value.request_id = "not-a-uuid".into(),
        |value: &mut ContextRequestIdentity| value.turn_id = "not-a-uuid".into(),
        |value: &mut ContextRequestIdentity| value.attempt = 0,
        |value: &mut ContextRequestIdentity| value.provider_id.clear(),
        |value: &mut ContextRequestIdentity| value.model = "m".repeat(129),
    ] {
        let mut invalid = measurement();
        mutate(&mut invalid.identity);
        assert!(ContextUsageRecord {
            last_measurement: Some(invalid),
            ..ContextUsageRecord::default()
        }
        .validate()
        .is_err());
    }
}

#[test]
fn unknown_counts_cannot_invent_tokens_or_a_source() {
    let mut invalid = measurement();
    invalid.input.coverage = ContextCountCoverage::Unknown;
    assert!(ContextUsageRecord {
        last_measurement: Some(invalid),
        ..ContextUsageRecord::default()
    }
    .validate()
    .is_err());
}

#[test]
fn capacity_never_falls_below_the_reported_count() {
    let mut invalid = measurement();
    invalid.input.capacity_tokens = Some(99);
    assert!(ContextUsageRecord {
        last_measurement: Some(invalid),
        ..ContextUsageRecord::default()
    }
    .validate()
    .is_err());
}

#[test]
fn measurements_require_a_complete_input_and_outputs_require_a_value() {
    let mut partial = measurement();
    partial.input.coverage = ContextCountCoverage::Partial;
    assert!(ContextUsageRecord {
        last_measurement: Some(partial),
        ..ContextUsageRecord::default()
    }
    .validate()
    .is_err());

    let mut missing = count(1, None);
    missing.tokens = None;
    missing.source = None;
    assert!(ContextUsageRecord {
        last_output: Some(ContextOutputSnapshot {
            identity: identity(),
            output: missing,
            updated_at: Utc::now(),
        }),
        ..ContextUsageRecord::default()
    }
    .validate()
    .is_err());
}

#[test]
fn invalidation_marks_only_the_preparation_stale() {
    let measurement = measurement();
    let mut record = ContextUsageRecord {
        current_preparation: Some(ContextPreparationSnapshot {
            identity: identity(),
            context_limit: Some(100_000),
            input: count(80, Some(80)),
            state: ContextPreparationState::InFlight,
            breakdown: None,
            transient_overhead_tokens: 0,
            updated_at: Utc::now(),
        }),
        last_measurement: Some(measurement.clone()),
        ..Default::default()
    };

    record.invalidate_preparation();

    assert_eq!(
        record.current_preparation.unwrap().state,
        ContextPreparationState::Stale
    );
    assert_eq!(record.last_measurement, Some(measurement));
}
