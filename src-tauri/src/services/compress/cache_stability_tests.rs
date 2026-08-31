use super::checkpoint_selection::{select, CheckpointSelectionLimits};

#[tokio::test]
async fn unchanged_reconstruction_and_tool_head_are_byte_identical() {
    let session = super::snapshot_tests::session();
    let canonical = vec![
        crate::services::agent_local::types_ollama::ChatMessage::system("stable system".into()),
    ];
    let tools = vec![
        serde_json::json!({
            "type": "function",
            "function": {"name": "alpha", "description": "A", "parameters": {"type": "object"}}
        }),
        serde_json::json!({
            "type": "function",
            "function": {"name": "beta", "description": "B", "parameters": {"type": "object"}}
        }),
    ];
    let snapshot = super::snapshot_tests::snapshot(&session)
        .with_runtime_context(canonical.clone(), tools.clone(), 40_000)
        .unwrap();
    let head_before =
        serde_json::to_vec(&(&snapshot.provider_tools, &snapshot.canonical_messages)).unwrap();

    let runtime = super::checkpoint_candidate_runtime::project(&snapshot, &session.messages);
    let head_after =
        serde_json::to_vec(&(&snapshot.provider_tools, &runtime[..canonical.len()])).unwrap();

    assert_eq!(head_before, head_after);
    assert_eq!(snapshot.provider_tools[0]["function"]["name"], "alpha");
    assert_eq!(snapshot.provider_tools[1]["function"]["name"], "beta");
}

#[test]
fn retained_messages_are_not_rewritten() {
    let session = super::snapshot_tests::session();
    let source = session.messages.last().unwrap();
    let source_bytes = serde_json::to_vec(source).unwrap();
    let selected = select(
        &session.messages,
        CheckpointSelectionLimits {
            recent_message_count: 8,
            tool_tokens: u32::MAX,
            tool_tokens_per_result: u32::MAX,
            max_tool_events: 100,
            total_tokens: u32::MAX,
        },
    )
    .unwrap();
    let retained = selected
        .messages
        .iter()
        .find(|message| message.message().id == source.id)
        .unwrap();

    assert_eq!(
        serde_json::to_vec(retained.message()).unwrap(),
        source_bytes
    );
}

#[test]
fn tool_result_excerpt_keeps_storage_lifecycle_untouched() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("tool-results/session");
    std::fs::create_dir_all(&directory).unwrap();
    let full_path = directory.join("result.txt");
    let full_content = "complete persisted result";
    std::fs::write(&full_path, full_content).unwrap();
    let modified_before = std::fs::metadata(&full_path).unwrap().modified().unwrap();
    let entries_before = std::fs::read_dir(&directory).unwrap().count();
    let mut message = super::snapshot_tests::session().messages[0].clone();
    message.role = "tool".into();
    message.content = format!(
        "{}\n[Résultat complet disponible : {}]",
        "preview ".repeat(5_000),
        full_path.display()
    );

    let excerpt = super::checkpoint_tools::excerpt_result(&message, 100);

    assert!(excerpt
        .content
        .contains(&full_path.to_string_lossy().to_string()));
    assert_eq!(std::fs::read_to_string(&full_path).unwrap(), full_content);
    assert_eq!(
        std::fs::metadata(&full_path).unwrap().modified().unwrap(),
        modified_before
    );
    assert_eq!(
        std::fs::read_dir(&directory).unwrap().count(),
        entries_before
    );
}
