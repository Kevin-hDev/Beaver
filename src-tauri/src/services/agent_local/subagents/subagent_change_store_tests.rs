use super::types_subagent_change::{SubagentChangeMeta, SubagentChangeStatus};
use chrono::{Duration, Utc};

#[tokio::test]
async fn evicts_oldest_terminal_change_when_store_is_full() {
    let dir = tempfile::tempdir().expect("temp store");
    let oldest = change(SubagentChangeStatus::Applied, 3);
    let pending = change(SubagentChangeStatus::Pending, 2);
    let incoming = change(SubagentChangeStatus::Pending, 1);

    save(&oldest, dir.path(), 2).await.expect("oldest");
    save(&pending, dir.path(), 2).await.expect("pending");
    save(&incoming, dir.path(), 2).await.expect("incoming");

    assert!(!entry(dir.path(), &oldest).exists());
    assert!(entry(dir.path(), &pending).exists());
    assert!(entry(dir.path(), &incoming).exists());
}

#[tokio::test]
async fn rejects_new_change_when_all_retained_changes_are_active() {
    let dir = tempfile::tempdir().expect("temp store");
    let first = change(SubagentChangeStatus::Pending, 2);
    let second = change(SubagentChangeStatus::Conflict, 1);
    let incoming = change(SubagentChangeStatus::Pending, 0);

    save(&first, dir.path(), 2).await.expect("first");
    save(&second, dir.path(), 2).await.expect("second");

    assert!(save(&incoming, dir.path(), 2).await.is_err());
    assert!(!entry(dir.path(), &incoming).exists());
}

async fn save(meta: &SubagentChangeMeta, dir: &std::path::Path, limit: usize) -> Result<(), String> {
    super::subagent_change_store::save_in_dir_for_test(meta, dir, limit).await
}

fn entry(dir: &std::path::Path, meta: &SubagentChangeMeta) -> std::path::PathBuf {
    dir.join(format!("{}.json", meta.child_session_id))
}

fn change(status: SubagentChangeStatus, age_minutes: i64) -> SubagentChangeMeta {
    let now = Utc::now() - Duration::minutes(age_minutes);
    SubagentChangeMeta {
        id: uuid::Uuid::new_v4().to_string(),
        child_session_id: uuid::Uuid::new_v4().to_string(),
        project_id: "project".into(),
        base_commit: "a".repeat(40),
        commit: "b".repeat(40),
        branch: format!("cl-go/subagent/{}", uuid::Uuid::new_v4()),
        target_branch: "main".into(),
        workspace_kind: super::types_subagent_change::SubagentWorkspaceKind::Git,
        changed_paths: Vec::new(),
        paths_truncated: false,
        status,
        created_at: now,
        updated_at: now,
        applied_commit: None,
    }
}
