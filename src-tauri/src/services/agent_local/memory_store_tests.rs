use super::*;
use crate::services::agent_local::memory_paths::MemoryLayout;

fn topic(id: &str, status: &str) -> String {
    format!(
        "---\n\
         id: {id}\n\
         scope: global\n\
         type: preference\n\
         status: {status}\n\
         title: Interface compacte\n\
         summary: Préférence pour une interface compacte.\n\
         created_at: 2026-07-24T20:00:00Z\n\
         updated_at: 2026-07-24T20:10:00Z\n\
         tags: [ui]\n\
         source: user\n\
         session_id: 019f951b-38a1-7882-bf2f-0784e266c911\n\
         ---\n\
         # Interface compacte\n\nUtiliser des contrôles compacts."
    )
}

#[tokio::test]
async fn topic_write_rebuilds_registry_and_summary() {
    let root = tempfile::tempdir().unwrap();
    let scope = MemoryLayout::at(root.path().join("memory")).global_scope();
    let id = uuid::Uuid::new_v4().to_string();
    let path = scope.topics_dir().join(format!("{id}.md"));

    write_topic(&scope, &path, &topic(&id, "confirmed"))
        .await
        .unwrap();

    let registry = tokio::fs::read_to_string(scope.registry_path())
        .await
        .unwrap();
    let summary = tokio::fs::read_to_string(scope.summary_path())
        .await
        .unwrap();
    assert!(registry.contains(&format!("topics/{id}.md")));
    assert!(summary.contains("Interface compacte"));
    assert!(summary.contains(path.to_str().unwrap()));
}

#[tokio::test]
async fn edit_detects_a_stale_source() {
    let root = tempfile::tempdir().unwrap();
    let scope = MemoryLayout::at(root.path().join("memory")).global_scope();
    let id = uuid::Uuid::new_v4().to_string();
    let path = scope.topics_dir().join(format!("{id}.md"));
    write_topic(&scope, &path, &topic(&id, "confirmed"))
        .await
        .unwrap();

    let error = edit_topic(&scope, &path, "texte absent", "nouveau")
        .await
        .unwrap_err();

    assert!(error.contains("Relisez"));
}

#[tokio::test]
async fn archived_topic_moves_out_of_active_indexes() {
    let root = tempfile::tempdir().unwrap();
    let scope = MemoryLayout::at(root.path().join("memory")).global_scope();
    let id = uuid::Uuid::new_v4().to_string();
    let path = scope.topics_dir().join(format!("{id}.md"));

    write_topic(&scope, &path, &topic(&id, "archived"))
        .await
        .unwrap();

    assert!(!path.exists());
    assert!(scope.archive_dir().join(format!("{id}.md")).exists());
    assert!(!tokio::fs::read_to_string(scope.summary_path())
        .await
        .unwrap()
        .contains("Interface compacte"));
}

#[tokio::test]
async fn concurrent_edits_do_not_overwrite_each_other() {
    let root = tempfile::tempdir().unwrap();
    let scope = MemoryLayout::at(root.path().join("memory")).global_scope();
    let id = uuid::Uuid::new_v4().to_string();
    let path = scope.topics_dir().join(format!("{id}.md"));
    write_topic(&scope, &path, &topic(&id, "confirmed"))
        .await
        .unwrap();

    let first = edit_topic(&scope, &path, "contrôles compacts", "boutons compacts");
    let second = edit_topic(&scope, &path, "contrôles compacts", "menus compacts");
    let (first, second) = tokio::join!(first, second);

    assert_ne!(first.is_ok(), second.is_ok());
    let saved = tokio::fs::read_to_string(path).await.unwrap();
    assert!(saved.contains("boutons compacts") || saved.contains("menus compacts"));
}

#[tokio::test]
async fn settings_archive_updates_metadata_and_active_indexes() {
    let root = tempfile::tempdir().unwrap();
    let scope = MemoryLayout::at(root.path().join("memory")).global_scope();
    let id = uuid::Uuid::new_v4().to_string();
    let path = scope.topics_dir().join(format!("{id}.md"));
    write_topic(&scope, &path, &topic(&id, "confirmed"))
        .await
        .unwrap();

    archive_topic(&scope, &path).await.unwrap();

    let archived = tokio::fs::read_to_string(scope.archive_dir().join(format!("{id}.md")))
        .await
        .unwrap();
    assert!(archived.contains("status: archived"));
    assert!(!scope.summary_path().exists() || !tokio::fs::read_to_string(scope.summary_path())
        .await
        .unwrap()
        .contains("Interface compacte"));
}

#[tokio::test]
async fn archive_collision_preserves_both_existing_files() {
    let root = tempfile::tempdir().unwrap();
    let scope = MemoryLayout::at(root.path().join("memory")).global_scope();
    let id = uuid::Uuid::new_v4().to_string();
    let path = scope.topics_dir().join(format!("{id}.md"));
    write_topic(&scope, &path, &topic(&id, "confirmed"))
        .await
        .unwrap();
    crate::services::private_store::ensure_private_dir_async(scope.archive_dir())
        .await
        .unwrap();
    let archived_path = scope.archive_dir().join(format!("{id}.md"));
    tokio::fs::write(&archived_path, "archive existante")
        .await
        .unwrap();

    assert!(archive_topic(&scope, &path).await.is_err());
    assert!(tokio::fs::read_to_string(&path)
        .await
        .unwrap()
        .contains("status: confirmed"));
    assert_eq!(
        tokio::fs::read_to_string(archived_path).await.unwrap(),
        "archive existante"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn archive_directory_symlink_is_rejected_before_writing() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let scope = MemoryLayout::at(root.path().join("memory")).global_scope();
    let id = uuid::Uuid::new_v4().to_string();
    let path = scope.topics_dir().join(format!("{id}.md"));
    write_topic(&scope, &path, &topic(&id, "confirmed"))
        .await
        .unwrap();
    std::os::unix::fs::symlink(outside.path(), scope.archive_dir()).unwrap();

    assert!(archive_topic(&scope, &path).await.is_err());
    assert!(path.exists());
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
}
