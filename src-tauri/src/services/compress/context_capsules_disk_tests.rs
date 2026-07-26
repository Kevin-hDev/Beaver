use super::*;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use tempfile::tempdir;

fn assistant(tool: &str, path: &str) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        tool_calls: Some(vec![ToolCallOllama {
            id: None,
            extra_content: None,
            function: ToolCallFunction {
                name: tool.to_string(),
                arguments: serde_json::json!({ "path": path }),
            },
        }]),
        ..Default::default()
    }
}

fn tool(content: &str) -> ChatMessage {
    ChatMessage {
        role: "tool".to_string(),
        content: content.to_string(),
        ..Default::default()
    }
}

fn user(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: content.to_string(),
        ..Default::default()
    }
}

#[tokio::test]
async fn read_then_edit_same_file_uses_final_disk_content_once() {
    let tmp = tempdir().unwrap();
    tokio::fs::write(tmp.path().join("a.rs"), "final content")
        .await
        .unwrap();
    let messages = vec![
        assistant("read_file", "a.rs"),
        tool("old content"),
        assistant("edit_file", "a.rs"),
        tool("Modifié: a.rs (ligne 1)"),
    ];
    let msg = compression_context_message(&messages, 200_000, tmp.path(), CompressionMode::Manual)
        .await
        .unwrap();
    assert_eq!(msg.content.matches("\n- ").count(), 1);
    assert!(msg.content.contains("final content"));
    assert!(!msg.content.contains("old content"));
}

#[tokio::test]
async fn write_file_reads_real_content_from_disk() {
    let tmp = tempdir().unwrap();
    tokio::fs::write(tmp.path().join("new.rs"), "created content")
        .await
        .unwrap();
    let messages = vec![assistant("write_file", "new.rs"), tool("Écrit: new.rs")];
    let msg = compression_context_message(&messages, 200_000, tmp.path(), CompressionMode::Manual)
        .await
        .unwrap();
    assert!(msg.content.contains("created content"));
    assert!(!msg.content.contains("Écrit: new.rs\n"));
}

#[tokio::test]
async fn manual_keeps_only_five_recent_files() {
    let tmp = tempdir().unwrap();
    let mut messages = Vec::new();
    for idx in 0..6 {
        let name = format!("f{idx}.rs");
        tokio::fs::write(tmp.path().join(&name), format!("content {idx}"))
            .await
            .unwrap();
        messages.push(assistant("read_file", &name));
        messages.push(tool("ignored cache"));
    }
    let msg = compression_context_message(&messages, 200_000, tmp.path(), CompressionMode::Manual)
        .await
        .unwrap();
    assert!(!msg.content.contains("f0.rs"));
    assert!(msg.content.contains("f1.rs"));
    assert!(msg.content.contains("f5.rs"));
}

#[tokio::test]
async fn auto_scans_only_since_request_start() {
    let tmp = tempdir().unwrap();
    tokio::fs::write(tmp.path().join("old.rs"), "old")
        .await
        .unwrap();
    tokio::fs::write(tmp.path().join("now.rs"), "now")
        .await
        .unwrap();
    let messages = vec![
        user("ancienne demande"),
        assistant("read_file", "old.rs"),
        tool("old cache"),
        user("nouvelle demande"),
        assistant("read_file", "now.rs"),
        tool("now cache"),
    ];
    let msg = compression_context_message(
        &messages,
        200_000,
        tmp.path(),
        CompressionMode::Auto {
            request_start_index: 3,
        },
    )
    .await
    .unwrap();
    assert!(!msg.content.contains("old.rs"));
    assert!(msg.content.contains("now.rs"));
}

#[tokio::test]
async fn auto_keeps_only_fifteen_recent_files() {
    let tmp = tempdir().unwrap();
    let mut messages = vec![user("demande courante")];
    for idx in 0..20 {
        let name = format!("auto{idx}.rs");
        tokio::fs::write(tmp.path().join(&name), format!("content {idx}"))
            .await
            .unwrap();
        messages.push(assistant("read_file", &name));
        messages.push(tool("ignored cache"));
    }
    let msg = compression_context_message(
        &messages,
        200_000,
        tmp.path(),
        CompressionMode::Auto {
            request_start_index: 0,
        },
    )
    .await
    .unwrap();
    assert!(!msg.content.contains("auto4.rs"));
    assert!(msg.content.contains("auto5.rs"));
    assert!(msg.content.contains("auto19.rs"));
}

#[tokio::test]
async fn unavailable_file_uses_marker_without_cached_content() {
    let tmp = tempdir().unwrap();
    let messages = vec![assistant("write_file", "gone.rs"), tool("Écrit: gone.rs")];
    let msg = compression_context_message(&messages, 200_000, tmp.path(), CompressionMode::Manual)
        .await
        .unwrap();
    assert!(msg.content.contains("[file unavailable"));
    assert!(!msg.content.contains("Écrit: gone.rs"));
}

#[tokio::test]
async fn binary_file_uses_marker() {
    let tmp = tempdir().unwrap();
    tokio::fs::write(tmp.path().join("bin.dat"), [0xff, 0xfe, 0xfd])
        .await
        .unwrap();
    let messages = vec![assistant("read_file", "bin.dat"), tool("binary cache")];
    let msg = compression_context_message(&messages, 200_000, tmp.path(), CompressionMode::Manual)
        .await
        .unwrap();
    assert!(msg.content.contains("[file unavailable"));
    assert!(!msg.content.contains("binary cache"));
}

#[tokio::test]
async fn large_file_is_truncated() {
    let tmp = tempdir().unwrap();
    tokio::fs::write(tmp.path().join("big.rs"), "x".repeat(40_000))
        .await
        .unwrap();
    let messages = vec![assistant("read_file", "big.rs"), tool("cache")];
    let msg = compression_context_message(&messages, 200_000, tmp.path(), CompressionMode::Manual)
        .await
        .unwrap();
    assert!(msg
        .content
        .contains("[content truncated for context budget]"));
}
