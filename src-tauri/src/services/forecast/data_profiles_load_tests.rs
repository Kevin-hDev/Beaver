use super::*;

#[tokio::test]
async fn invalid_and_missing_profiles_have_distinct_failures() {
    assert_eq!(
        load_profile_classified("not-a-session", "not-a-uuid")
            .await
            .unwrap_err(),
        DataProfileLoadError::InvalidId
    );
    assert_eq!(
        classify_io(std::io::Error::from(std::io::ErrorKind::NotFound)),
        DataProfileLoadError::NotFound
    );
}

#[tokio::test]
async fn claims_a_real_pre_scope_profile_for_exactly_one_workspace() {
    let first = create_session("Legacy profile owner").await;
    let second = create_session("Other workspace").await;
    let id = uuid::Uuid::new_v4().to_string();
    let mut fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/forecast-data-profile-v1.json"))
            .expect("historical profile fixture");
    fixture["profile"]["id"] = serde_json::Value::String(id.clone());
    let legacy = crate::services::paths::data_file_for_write(
        "forecast-data-profiles",
        &format!("{id}.json"),
    )
    .await
    .expect("legacy path");
    crate::services::private_store::atomic_write_async(
        legacy,
        serde_json::to_vec_pretty(&fixture).expect("fixture bytes"),
    )
    .await
    .expect("seed historical profile");

    let claimed = load_profile_classified(&first.id, &id)
        .await
        .expect("claim historical profile");
    let rejected = load_profile_classified(&second.id, &id).await;

    assert_eq!(claimed.id, id);
    assert!(matches!(rejected, Err(DataProfileLoadError::NotFound)));
    cleanup(&first.id, &id).await;
    cleanup(&second.id, &id).await;
}

async fn create_session(name: &str) -> crate::services::agent_local::types_session::AgentSession {
    crate::services::agent_local::session_store::create_full(name, "model", "provider", false, None)
        .await
        .expect("create session")
}

async fn cleanup(session_id: &str, profile_id: &str) {
    if let Ok(workspace) = crate::services::workspace_scope::resolve(session_id).await {
        if let Ok(path) =
            super::super::data_profiles::profile_path_for_read(&workspace, profile_id).await
        {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
    let _ = crate::services::agent_local::session_store::delete_one(session_id).await;
    if let Ok(path) = super::super::data_profiles::legacy_profile_path_for_read(profile_id).await {
        let _ = tokio::fs::remove_file(path).await;
    }
}
