use super::*;

fn project_topic(id: &str) -> String {
    format!(
        "---\n\
         id: {id}\n\
         scope: project\n\
         type: project\n\
         status: confirmed\n\
         title: Convention du projet\n\
         summary: Convention durable du projet.\n\
         created_at: 2026-07-25T00:00:00Z\n\
         updated_at: 2026-07-25T00:00:00Z\n\
         tags: [project]\n\
         source: user\n\
         session_id: 019f951b-38a1-7882-bf2f-0784e266c911\n\
         ---\n\
         Convention durable."
    )
}

fn setup() -> (tempfile::TempDir, MemoryLayout, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("Projects").join("CL-GO-DASH");
    std::fs::create_dir_all(&project).unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    (root, layout, project)
}

fn legacy_scope(layout: &MemoryLayout, project: &Path) -> MemoryScope {
    let identity = project_identity(project).unwrap();
    MemoryScope {
        id: identity.legacy_id.clone(),
        label: identity.label,
        root: layout.root().join("projects").join(identity.legacy_id),
    }
}

fn current_scope(layout: &MemoryLayout, project: &Path) -> MemoryScope {
    let identity = project_identity(project).unwrap();
    MemoryScope {
        id: identity.id.clone(),
        label: identity.label,
        root: layout.root().join("projects").join(identity.id),
    }
}

fn canonical_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

#[tokio::test]
async fn legacy_folder_is_renamed_and_indexes_are_rebuilt() {
    let (_root, layout, project) = setup();
    let legacy = legacy_scope(&layout, &project);
    let topic_id = uuid::Uuid::new_v4().to_string();
    let topic_path = legacy.topics_dir().join(format!("{topic_id}.md"));
    super::super::memory_store::write_topic(
        &legacy,
        &topic_path,
        &project_topic(&topic_id),
    )
    .await
    .unwrap();
    assert!(
        tokio::fs::read_to_string(legacy.summary_path())
            .await
            .unwrap()
            .contains(&canonical_path(&legacy.root))
    );

    let resolved = resolve(&layout, &project).await.unwrap();
    let summary = tokio::fs::read_to_string(resolved.summary_path())
        .await
        .unwrap();

    assert!(!legacy.root.exists());
    assert!(resolved.root.exists());
    assert!(summary.contains(&canonical_path(&resolved.root)));
    assert!(!resolved.root.join(PENDING_MARKER).exists());
}

#[tokio::test]
async fn resolving_an_unused_project_does_not_create_memory() {
    let (_root, layout, project) = setup();

    let resolved = resolve(&layout, &project).await.unwrap();

    assert!(!resolved.root.exists());
    assert!(!layout.root().exists());
}

#[tokio::test]
async fn collision_preserves_both_folders_and_fails_closed() {
    let (_root, layout, project) = setup();
    let legacy = legacy_scope(&layout, &project);
    let current = current_scope(&layout, &project);
    legacy.ensure().await.unwrap();
    current.ensure().await.unwrap();

    assert!(resolve(&layout, &project).await.is_err());
    assert!(legacy.root.exists());
    assert!(current.root.exists());
}

#[tokio::test]
async fn pending_migration_is_resumed_after_an_interruption() {
    let (_root, layout, project) = setup();
    let legacy = legacy_scope(&layout, &project);
    let current = current_scope(&layout, &project);
    let topic_id = uuid::Uuid::new_v4().to_string();
    let topic_path = legacy.topics_dir().join(format!("{topic_id}.md"));
    super::super::memory_store::write_topic(
        &legacy,
        &topic_path,
        &project_topic(&topic_id),
    )
    .await
    .unwrap();
    ensure_pending_marker(&legacy).await.unwrap();
    tokio::fs::rename(&legacy.root, &current.root).await.unwrap();

    let resolved = resolve(&layout, &project).await.unwrap();
    let summary = tokio::fs::read_to_string(resolved.summary_path())
        .await
        .unwrap();

    assert!(summary.contains(&canonical_path(&resolved.root)));
    assert!(!resolved.root.join(PENDING_MARKER).exists());
}

#[cfg(unix)]
#[tokio::test]
async fn legacy_symlink_is_rejected_without_touching_its_target() {
    use std::os::unix::fs::symlink;

    let (_root, layout, project) = setup();
    let legacy = legacy_scope(&layout, &project);
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(legacy.root.parent().unwrap()).unwrap();
    symlink(outside.path(), &legacy.root).unwrap();

    assert!(resolve(&layout, &project).await.is_err());
    assert!(!outside.path().join(PENDING_MARKER).exists());
}
