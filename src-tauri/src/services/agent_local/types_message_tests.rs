use super::{AgentMessage, SavedSegment};
use crate::services::agent_local::types_stream::TokenPhase;

#[test]
fn agent_message_accepts_missing_work_duration() {
    let msg: AgentMessage = serde_json::from_value(serde_json::json!({
        "id": "m1", "role": "assistant", "content": "ok", "files": [],
        "timestamp": "2026-07-01T12:00:00Z", "tokens": 0
    }))
    .unwrap();

    assert_eq!(msg.work_duration_ms, None);
}

#[test]
fn agent_message_serializes_work_duration_when_present() {
    let msg: AgentMessage = serde_json::from_value(serde_json::json!({
        "id": "m1", "role": "assistant", "content": "ok", "files": [],
        "timestamp": "2026-07-01T12:00:00Z", "tokens": 0,
        "work_duration_ms": 266000
    }))
    .unwrap();
    let saved = serde_json::to_value(msg).unwrap();

    assert_eq!(saved["work_duration_ms"], 266000);
}

#[test]
fn agent_message_persists_stream_group_metadata() {
    let msg: AgentMessage = serde_json::from_value(serde_json::json!({
        "id": "m-stream", "role": "assistant", "content": "travail", "files": [],
        "timestamp": "2026-07-12T12:00:00Z",
        "stream_run_id": "7c8e3a14-8811-4d88-9a54-d234547d8d22",
        "stream_part": "checkpoint"
    }))
    .unwrap();

    let saved = serde_json::to_value(msg).unwrap();
    assert_eq!(saved["stream_run_id"], "7c8e3a14-8811-4d88-9a54-d234547d8d22");
    assert_eq!(saved["stream_part"], "checkpoint");
}

#[test]
fn agent_message_rejects_incomplete_or_invalid_stream_metadata() {
    let incomplete: AgentMessage = serde_json::from_value(serde_json::json!({
        "id": "m-stream", "role": "assistant", "content": "travail",
        "files": [], "timestamp": "2026-07-12T12:00:00Z",
        "stream_run_id": "7c8e3a14-8811-4d88-9a54-d234547d8d22"
    }))
    .unwrap();
    assert!(incomplete.validate_stream_metadata().is_err());

    let invalid: AgentMessage = serde_json::from_value(serde_json::json!({
        "id": "m-stream", "role": "assistant", "content": "travail",
        "files": [], "timestamp": "2026-07-12T12:00:00Z",
        "stream_run_id": "7c8e3a14-8811-4d88-9a54-d234547d8d22",
        "stream_part": "other"
    }))
    .unwrap();
    assert!(invalid.validate_stream_metadata().is_err());
}

#[test]
fn saved_segment_accepts_phase() {
    let segment: SavedSegment = serde_json::from_value(serde_json::json!({
        "tools": [], "content": "partial", "phase": "work"
    }))
    .unwrap();

    assert!(matches!(segment.phase, Some(TokenPhase::Work)));
}

#[test]
fn agent_message_rejects_unbounded_file_change_history() {
    let mut msg: AgentMessage = serde_json::from_value(serde_json::json!({
        "id": "m1", "role": "assistant", "content": "ok",
        "files": [], "timestamp": "2026-07-01T12:00:00Z",
        "tool_activities": [{
            "name": "bash", "summary": "test",
            "file_changes": (0..501).map(|index| serde_json::json!({
                "path": format!("/repo/{index}.txt"),
                "status": "added", "additions": 1, "deletions": 0
            })).collect::<Vec<_>>()
        }]
    }))
    .unwrap();

    assert!(msg.validate_stream_metadata().is_err());
    msg.tool_activities.as_mut().unwrap()[0].file_changes.truncate(500);
    assert!(msg.validate_stream_metadata().is_ok());
}

#[test]
fn agent_message_rejects_unbounded_affected_path_history() {
    let message: AgentMessage = serde_json::from_value(serde_json::json!({
        "id": "m1", "role": "assistant", "content": "ok",
        "files": [], "timestamp": "2026-07-01T12:00:00Z",
        "tool_activities": [{
            "name": "bash", "summary": "test",
            "affected_paths": (0..501)
                .map(|index| format!("/repo/{index}.txt"))
                .collect::<Vec<_>>()
        }]
    }))
    .unwrap();

    assert!(message.validate_stream_metadata().is_err());
}

#[test]
fn agent_message_validates_memory_tool_metadata() {
    let make_message = |domain: &str, resolved_path: &str| {
        serde_json::from_value::<AgentMessage>(serde_json::json!({
            "id": "m-memory", "role": "assistant", "content": "ok",
            "files": [], "timestamp": "2026-07-01T12:00:00Z",
            "tool_activities": [{
                "name": "read_file", "summary": "MEMORY.md",
                "domain": domain, "resolved_path": resolved_path
            }]
        }))
        .unwrap()
    };

    assert!(make_message("memory", "/memory/global/MEMORY.md")
        .validate_stream_metadata()
        .is_ok());
    assert!(make_message("other", "/memory/global/MEMORY.md")
        .validate_stream_metadata()
        .is_err());
    assert!(make_message("memory", "bad\0path")
        .validate_stream_metadata()
        .is_err());
}

#[test]
fn agent_message_validates_structured_tool_results() {
    let make_message = |status: &str, is_error: bool, code: &str| {
        serde_json::from_value::<AgentMessage>(serde_json::json!({
            "id": "m-tool", "role": "assistant", "content": "",
            "files": [], "timestamp": "2026-07-01T12:00:00Z",
            "tool_activities": [{
                "name": "bash", "summary": "false", "result": "failed",
                "is_error": is_error,
                "result_meta": {
                    "status": status,
                    "error": {
                        "code": code, "category": "execution", "retryable": false
                    },
                    "warnings": [], "truncated": false
                }
            }]
        }))
        .unwrap()
    };

    assert!(make_message("error", true, "shell_exit_nonzero")
        .validate_stream_metadata()
        .is_ok());
    assert!(make_message("error", false, "shell_exit_nonzero")
        .validate_stream_metadata()
        .is_err());
    assert!(make_message("error", true, "INVALID CODE")
        .validate_stream_metadata()
        .is_err());
}

#[test]
fn agent_message_rejects_unsafe_tool_metadata_text() {
    let message = serde_json::from_value::<AgentMessage>(serde_json::json!({
        "id": "m-tool", "role": "assistant", "content": "",
        "files": [], "timestamp": "2026-07-01T12:00:00Z",
        "tool_activities": [{
            "name": "grep", "summary": "search", "result": "partial",
            "is_error": false,
            "result_meta": {
                "status": "partial",
                "warnings": ["safe\u{202e}text"],
                "truncated": false
            }
        }]
    }))
    .unwrap();

    assert!(message.validate_stream_metadata().is_err());
}

fn message_with_artifacts(artifacts: serde_json::Value) -> AgentMessage {
    serde_json::from_value(serde_json::json!({
        "id": "m-artifact", "role": "assistant", "content": "ok",
        "files": [], "timestamp": "2026-07-01T12:00:00Z",
        "tool_activities": [{
            "name": "extension_tool", "summary": "artifact",
            "artifacts": artifacts
        }]
    }))
    .unwrap()
}

fn artifact_value() -> serde_json::Value {
    serde_json::json!({
        "name": "preview.png",
        "mime_type": "image/png",
        "bytes": 8,
        "sha256": "a".repeat(64),
        "purpose": "preview",
        "source": {
            "kind": "extension_resource",
            "resource_id": "extension:sample:preview",
            "catalog_fingerprint": "b".repeat(64)
        }
    })
}

#[test]
fn agent_message_accepts_missing_empty_and_exact_artifact_limits() {
    let missing = message_with_artifacts(serde_json::json!([]));
    assert!(missing.validate_stream_metadata().is_ok());

    let exact = message_with_artifacts(serde_json::Value::Array(vec![
        artifact_value();
        super::super::tool_artifact_record::MAX_ARTIFACTS_PER_TOOL
    ]));
    assert!(exact.validate_stream_metadata().is_ok());
}

#[test]
fn agent_message_rejects_excess_or_malformed_artifact_metadata() {
    let too_many = message_with_artifacts(serde_json::Value::Array(vec![
        artifact_value();
        super::super::tool_artifact_record::MAX_ARTIFACTS_PER_TOOL + 1
    ]));
    assert!(too_many.validate_stream_metadata().is_err());

    for (pointer, value) in [
        ("/sha256", serde_json::json!("not-a-sha")),
        ("/bytes", serde_json::json!(u64::MAX)),
        ("/source/resource_id", serde_json::json!("extension:missing")),
    ] {
        let mut artifact = artifact_value();
        *artifact.pointer_mut(pointer).expect("fixture field") = value;
        let malformed = message_with_artifacts(serde_json::json!([artifact]));
        assert!(malformed.validate_stream_metadata().is_err(), "{pointer}");
    }
}
