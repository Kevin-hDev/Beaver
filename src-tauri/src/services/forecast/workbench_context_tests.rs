use super::super::{
    input_data::InputSnapshot,
    types::{ForecastResult, InputSummary, Quantiles},
};
use super::{bounded_name, next_context, validate_ids, ForecastWorkbenchContext};

const SESSION: &str = "550e8400-e29b-41d4-a716-446655440000";
const ANALYSIS: &str = "123e4567-e89b-12d3-a456-426614174000";

#[test]
fn context_ids_reject_traversal() {
    assert!(validate_ids("../session", Some(ANALYSIS)).is_err());
    assert!(validate_ids(SESSION, Some("../analysis")).is_err());
}

#[test]
fn context_revision_increments_and_keeps_bounded_ids() {
    let first = next_context(None, SESSION.into(), Some(ANALYSIS.into()));
    let second = next_context(Some(&first), SESSION.into(), None);

    assert_eq!(first.revision, 1);
    assert_eq!(second.revision, 2);
    assert_eq!(second.analysis_id, None);
}

#[test]
fn revision_overflow_restarts_without_panicking() {
    let current = ForecastWorkbenchContext {
        session_id: SESSION.into(),
        analysis_id: None,
        revision: u64::MAX,
    };

    assert_eq!(
        next_context(Some(&current), SESSION.into(), None).revision,
        1
    );
}

#[test]
fn context_names_are_sanitized_and_bounded() {
    let name = format!("{}\nsecret", "é".repeat(140));
    let bounded = bounded_name(name);

    assert_eq!(bounded.chars().count(), 120);
    assert!(!bounded.contains('\n'));
}

#[tokio::test]
async fn current_session_can_open_a_historical_analysis() {
    let session = crate::services::agent_local::session_store::create_with_flags(
        "Session active",
        "model",
        "provider",
        false,
    )
    .await
    .expect("create session");
    let mut analysis = historical_analysis();
    super::super::storage::save(&mut analysis)
        .await
        .expect("save analysis");

    let snapshot = super::set(session.id.clone(), Some(analysis.id.clone()))
        .await
        .expect("open historical analysis");

    assert_eq!(snapshot.context.session_id, session.id);
    assert_eq!(
        snapshot.context.analysis_id.as_deref(),
        Some(analysis.id.as_str())
    );
}

fn historical_analysis() -> ForecastResult {
    ForecastResult {
        schema_version: crate::services::forecast::types::CURRENT_SCHEMA_VERSION,
        revision: crate::services::forecast::types::default_revision(),
        id: uuid::Uuid::new_v4().to_string(),
        name: "Analyse historique".into(),
        target_column: "value".into(),
        created_at: "2026-07-21T00:00:00Z".into(),
        session_id: Some(uuid::Uuid::new_v4().to_string()),
        model: "model".into(),
        provider: "provider".into(),
        horizon: 1,
        frequency: "D".into(),
        confidence_level: 0.9,
        input_summary: InputSummary {
            points: 0,
            start: String::new(),
            end: String::new(),
        },
        input_data: InputSnapshot::default(),
        data_profile: None,
        predictions: vec![],
        quantiles: Quantiles {
            q10: vec![],
            q50: vec![],
            q90: vec![],
        },
        covariates_used: vec![],
        metrics: None,
        evaluation: None,
        advanced_analytics: None,
        ensemble: None,
        annotations: vec![],
        scenarios: vec![],
        provenance: Default::default(),
    }
}
