use super::*;

#[tokio::test]
async fn project_metadata_is_bounded_without_loading_topic_contents() {
    let root = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let scope = MemoryScope {
        id: "a".repeat(24),
        label: "Projet test".into(),
        root: layout.root().join("projects").join("a".repeat(24)),
    };
    scope.ensure().await.unwrap();
    for index in 0..(super::super::memory_types::MAX_TOPICS_PER_SCOPE + 4) {
        tokio::fs::write(
            scope.topics_dir().join(format!("{index:04}.md")),
            "contenu non parsé",
        )
        .await
        .unwrap();
    }

    let overview = scope_metadata(&scope).await.unwrap();

    assert_eq!(
        overview.topic_count,
        super::super::memory_types::MAX_TOPICS_PER_SCOPE
    );
    assert!(overview.topics.is_empty());
    assert!(!overview.topics_loaded);
    assert!(overview.total_bytes > 0);
}
