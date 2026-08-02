use super::serialize_prompt_metadata;
use crate::services::agent_local::types_subagent_change::{
    SubagentChangeMeta, SubagentChangeStatus, SubagentWorkspaceKind,
};
use chrono::Utc;
use serde_json::json;

#[test]
fn emitted_metadata_uses_the_exact_tool_argument_names() {
    let change = pending_change();
    let encoded = serialize_prompt_metadata(&change).expect("serialize metadata");
    let metadata: serde_json::Value = serde_json::from_str(&encoded).expect("parse metadata");

    assert_eq!(metadata["subagent_id"], change.child_session_id);
    assert_eq!(metadata["change_id"], change.id);
    assert_eq!(metadata["child_session_id"], metadata["subagent_id"]);
    assert_eq!(metadata["id"], metadata["change_id"]);

    let args = json!({
        "subagent_id": metadata["subagent_id"],
        "change_id": metadata["change_id"],
    });
    for tool in [
        "inspect_subagent_changes",
        "apply_subagent_changes",
        "discard_subagent_changes",
    ] {
        assert!(crate::services::agent_local::tool_validate::validate(tool, &args).is_ok());
    }
}

fn pending_change() -> SubagentChangeMeta {
    let now = Utc::now();
    SubagentChangeMeta {
        id: uuid::Uuid::new_v4().to_string(),
        child_session_id: uuid::Uuid::new_v4().to_string(),
        project_id: "project".into(),
        base_commit: "a".repeat(40),
        commit: "b".repeat(40),
        branch: "codex/subagent-change".into(),
        target_branch: "main".into(),
        workspace_kind: SubagentWorkspaceKind::Git,
        changed_paths: Vec::new(),
        paths_truncated: false,
        status: SubagentChangeStatus::Pending,
        created_at: now,
        updated_at: now,
        applied_commit: None,
    }
}
