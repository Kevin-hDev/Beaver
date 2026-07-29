use super::*;

#[tokio::test]
async fn project_ids_are_stable_and_isolated() {
    let root = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));

    let one = layout.project_scope_ready(first.path()).await.unwrap();
    let same = layout.project_scope_ready(first.path()).await.unwrap();
    let other = layout.project_scope_ready(second.path()).await.unwrap();

    assert_eq!(one.id, same.id);
    assert_ne!(one.id, other.id);
    assert!(valid_project_id(&one.id));
    assert!(one.id.len() <= super::super::memory_project_id::MAX_PROJECT_FOLDER_ID_BYTES);
}

#[tokio::test]
async fn only_global_and_active_project_are_accessible() {
    let root = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let active = layout.project_scope_ready(project.path()).await.unwrap();
    let foreign = layout.project_scope_ready(other.path()).await.unwrap();

    assert!(layout
        .scope_for_tool_path(
            active.topics_dir().join("x.md").to_str().unwrap(),
            project.path(),
        )
        .await
        .unwrap()
        .is_some());
    assert!(layout
        .scope_for_tool_path(
            foreign.topics_dir().join("x.md").to_str().unwrap(),
            project.path(),
        )
        .await
        .is_err());
}

#[tokio::test]
async fn traversal_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let path = layout.root().join("global/../global/MEMORY.md");

    assert!(layout
        .scope_for_tool_path(path.to_str().unwrap(), root.path())
        .await
        .is_err());
}

#[tokio::test]
async fn normal_project_paths_bypass_memory_validation() {
    let root = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));

    assert!(layout
        .scope_for_tool_path(".", root.path())
        .await
        .unwrap()
        .is_none());
    assert!(layout
        .scope_for_tool_path("./src", root.path())
        .await
        .unwrap()
        .is_none());
}

#[cfg(unix)]
#[tokio::test]
async fn a_scope_symlink_cannot_escape_the_memory_root() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    std::fs::create_dir_all(layout.root()).unwrap();
    symlink(outside.path(), layout.global_scope().root).unwrap();

    assert!(layout.global_scope().ensure().await.is_err());
    assert!(!outside.path().join("topics").exists());
}

#[tokio::test]
async fn management_accepts_only_existing_topics_in_managed_scopes() {
    let root = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let scope = layout.global_scope();
    scope.ensure().await.unwrap();
    let id = uuid::Uuid::new_v4();
    let topic = scope.topics_dir().join(format!("{id}.md"));
    tokio::fs::write(&topic, "test").await.unwrap();

    assert!(layout.management_topic(topic.to_str().unwrap()).is_ok());
    assert!(layout
        .management_topic(root.path().join("outside.md").to_str().unwrap())
        .is_err());
}
