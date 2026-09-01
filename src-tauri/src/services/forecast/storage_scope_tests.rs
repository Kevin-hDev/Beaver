use super::*;
use crate::services::forecast::input_data::InputSnapshot;
use crate::services::forecast::types::{InputSummary, Quantiles};

#[tokio::test]
async fn deleted_legacy_owner_becomes_unassigned_without_breaking_other_lists() {
    let current = create_session("Current").await;
    let owner = create_session("Deleted owner").await;
    let mut analysis = analysis(ForecastWorkspace::Legacy, Some(owner.id.clone()));
    super::super::storage::save(&mut analysis)
        .await
        .expect("seed legacy analysis");
    crate::services::agent_local::session_store::delete_one(&owner.id)
        .await
        .expect("delete owner");

    let visible = list_for_session(&current.id)
        .await
        .expect("an orphan must not poison the list");
    let unassigned = list_unassigned_for_session(&current.id)
        .await
        .expect("list unassigned");

    assert!(visible.iter().all(|entry| entry.id != analysis.id));
    assert!(unassigned.iter().any(|entry| entry.id == analysis.id));
    cleanup(&current.id, &analysis.id).await;
}

#[tokio::test]
async fn releasing_a_project_keeps_its_analyses_recoverable() {
    let current = create_session("Current").await;
    let workspace = ForecastWorkspace::Project("deleted-project".into());
    let mut analysis = analysis(workspace.clone(), Some(current.id.clone()));
    seed_existing_analysis(&mut analysis).await;

    release_workspace(&workspace, std::slice::from_ref(&current.id))
        .await
        .expect("release project resources");

    let stored = super::super::storage::load(&analysis.id)
        .await
        .expect("reload analysis");
    assert_eq!(stored.workspace, ForecastWorkspace::Legacy);
    assert_eq!(stored.session_id, None);
    cleanup(&current.id, &analysis.id).await;
}

#[tokio::test]
async fn deleting_a_root_session_releases_its_scoped_analysis_before_removal() {
    let owner = create_session("Deleted root").await;
    let mut analysis = analysis(
        ForecastWorkspace::Session(owner.id.clone()),
        Some(owner.id.clone()),
    );
    super::super::storage::save(&mut analysis)
        .await
        .expect("seed session analysis");

    crate::services::agent_local::session_store::delete(&owner.id)
        .await
        .expect("delete owner family");

    let stored = super::super::storage::load(&analysis.id)
        .await
        .expect("analysis remains readable");
    assert_eq!(stored.workspace, ForecastWorkspace::Legacy);
    assert_eq!(stored.session_id, None);
    let _ = super::super::storage::delete(&analysis.id).await;
}

#[tokio::test]
async fn deleting_a_root_session_releases_its_profile_before_removal() {
    let owner = create_session("Deleted profile owner").await;
    let workspace = ForecastWorkspace::Session(owner.id.clone());
    let id = uuid::Uuid::new_v4().to_string();
    let mut fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/forecast-data-profile-v1.json"))
            .expect("historical fixture");
    fixture["profile"]["id"] = serde_json::Value::String(id.clone());
    fixture["workspace"] = serde_json::to_value(&workspace).expect("workspace");
    let source = super::super::data_profiles::profile_path_for_write(&workspace, &id)
        .await
        .expect("profile path");
    crate::services::private_store::atomic_write_async(
        source,
        serde_json::to_vec_pretty(&fixture).expect("profile bytes"),
    )
    .await
    .expect("seed profile");

    crate::services::agent_local::session_store::delete(&owner.id)
        .await
        .expect("delete owner family");

    assert!(
        super::super::data_profiles::profile_path_for_read(&workspace, &id)
            .await
            .is_err()
    );
    let legacy = super::super::data_profiles::legacy_profile_path_for_read(&id)
        .await
        .expect("legacy profile");
    let stored = super::super::data_profiles_load::read_stored(&legacy, &id)
        .await
        .expect("read released profile");
    assert_eq!(stored.workspace, ForecastWorkspace::Legacy);
    let _ = tokio::fs::remove_file(legacy).await;
}

async fn create_session(name: &str) -> crate::services::agent_local::types_session::AgentSession {
    crate::services::agent_local::session_store::create_full(name, "model", "provider", false, None)
        .await
        .expect("create session")
}

fn analysis(workspace: ForecastWorkspace, session_id: Option<String>) -> ForecastResult {
    ForecastResult {
        schema_version: crate::services::forecast::types::CURRENT_SCHEMA_VERSION,
        revision: crate::services::forecast::types::default_revision(),
        id: uuid::Uuid::new_v4().to_string(),
        name: "Lifecycle analysis".into(),
        target_column: "value".into(),
        created_at: "2026-09-01T00:00:00Z".into(),
        workspace,
        session_id,
        model: "model".into(),
        provider: "provider".into(),
        horizon: 1,
        frequency: "D".into(),
        confidence_level: 0.9,
        input_summary: InputSummary {
            points: 1,
            start: String::new(),
            end: String::new(),
        },
        input_data: InputSnapshot::default(),
        data_profile: None,
        predictions: Vec::new(),
        quantiles: Quantiles {
            q10: Vec::new(),
            q50: Vec::new(),
            q90: Vec::new(),
        },
        covariates_used: Vec::new(),
        metrics: None,
        evaluation: None,
        advanced_analytics: None,
        ensemble: None,
        annotations: Vec::new(),
        scenarios: Vec::new(),
        provenance: Default::default(),
    }
}

async fn cleanup(session_id: &str, analysis_id: &str) {
    let _ = super::super::storage::delete(analysis_id).await;
    let _ = crate::services::agent_local::session_store::delete_one(session_id).await;
}

async fn seed_existing_analysis(analysis: &mut ForecastResult) {
    let target = super::super::storage_paths::analysis_path_for_write(&analysis.id)
        .await
        .expect("analysis path");
    crate::services::private_store::atomic_write_async(
        target,
        serde_json::to_vec_pretty(analysis).expect("analysis bytes"),
    )
    .await
    .expect("seed analysis file");
    super::super::storage_index::upsert(analysis.to_meta())
        .await
        .expect("seed analysis index");
}
