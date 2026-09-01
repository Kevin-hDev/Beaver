use super::*;

#[tokio::test]
async fn release_refuses_to_overwrite_another_workspaces_marker() {
    let id = uuid::Uuid::new_v4().to_string();
    let source_workspace = WorkspaceScope::Project("project-a".into());
    let other_workspace = WorkspaceScope::Session(uuid::Uuid::new_v4().to_string());
    seed(&id, &source_workspace, false).await;
    seed(&id, &other_workspace, true).await;

    let result = stage_release(&source_workspace).await;
    let marker = read_stored(
        &legacy_profile_path_for_read(&id).await.expect("marker"),
        &id,
    )
    .await
    .expect("read marker");

    assert!(result.is_err());
    assert_eq!(marker.workspace, other_workspace);
    cleanup(&id, &source_workspace).await;
}

#[tokio::test]
async fn release_moves_a_scoped_profile_to_the_recoverable_legacy_store() {
    let id = uuid::Uuid::new_v4().to_string();
    let workspace = WorkspaceScope::Project("project-release".into());
    seed(&id, &workspace, false).await;

    let ids = stage_release(&workspace).await.expect("stage release");
    commit_release(&workspace, &ids)
        .await
        .expect("commit release");

    assert!(profile_path_for_read(&workspace, &id).await.is_err());
    let legacy = read_stored(
        &legacy_profile_path_for_read(&id)
            .await
            .expect("legacy profile"),
        &id,
    )
    .await
    .expect("read legacy profile");
    assert_eq!(legacy.workspace, WorkspaceScope::Legacy);
    cleanup(&id, &workspace).await;
}

#[tokio::test]
async fn unrelated_json_does_not_block_a_profile_release() {
    let id = uuid::Uuid::new_v4().to_string();
    let workspace = WorkspaceScope::Project("project-with-extra-file".into());
    seed(&id, &workspace, false).await;
    let directory = crate::services::paths::data_dir()
        .join(profile_directory(&workspace).expect("profile directory"));
    let unrelated = directory.join("not-a-profile.json");
    crate::services::private_store::atomic_write_async(unrelated.clone(), b"{}".to_vec())
        .await
        .expect("seed unrelated file");

    let ids = stage_release(&workspace).await.expect("stage release");
    commit_release(&workspace, &ids)
        .await
        .expect("commit release");

    assert!(tokio::fs::try_exists(&unrelated)
        .await
        .expect("inspect extra file"));
    let _ = tokio::fs::remove_file(unrelated).await;
    cleanup(&id, &workspace).await;
}

async fn seed(id: &str, workspace: &WorkspaceScope, legacy: bool) {
    let mut fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/forecast-data-profile-v1.json"))
            .expect("historical fixture");
    fixture["profile"]["id"] = serde_json::Value::String(id.to_string());
    fixture["workspace"] = serde_json::to_value(workspace).expect("workspace");
    let path = if legacy {
        legacy_profile_path_for_write(id)
            .await
            .expect("legacy path")
    } else {
        profile_path_for_write(workspace, id)
            .await
            .expect("scoped path")
    };
    crate::services::private_store::atomic_write_async(
        path,
        serde_json::to_vec_pretty(&fixture).expect("fixture bytes"),
    )
    .await
    .expect("seed profile");
}

async fn cleanup(id: &str, workspace: &WorkspaceScope) {
    if let Ok(path) = profile_path_for_read(workspace, id).await {
        let _ = tokio::fs::remove_file(path).await;
    }
    if let Ok(path) = legacy_profile_path_for_read(id).await {
        let _ = tokio::fs::remove_file(path).await;
    }
}
