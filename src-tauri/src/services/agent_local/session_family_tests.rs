use super::*;
use chrono::Utc;

fn meta(id: &str, parent: Option<&str>, clone_parent: Option<&str>) -> AgentSessionMeta {
    AgentSessionMeta {
        id: id.into(),
        name: id.into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        message_count: 0,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        parent_session_id: parent.map(str::to_string),
        subagent_type: None,
        subagent_status: None,
        subagent_run_id: None,
        subagent_description: None,
        subagent_color_key: None,
        subagent_summary: None,
        subagent_last_activity: None,
        clone_parent_session_id: clone_parent.map(str::to_string),
        clone_parent_message_id: None,
        clone_mode: None,
        clone_root_session_id: None,
        git_branch: None,
    }
}

#[test]
fn archive_family_targets_parent_and_children() {
    let metas = vec![
        meta("root", None, None),
        meta("clone", None, Some("root")),
        meta("sub", Some("root"), None),
        meta("nested", Some("clone"), None),
    ];

    assert_eq!(
        archive_targets(&metas, "root"),
        ["root", "clone", "sub", "nested"]
    );
}

#[test]
fn restore_with_parent_targets_parent_before_child() {
    let metas = vec![
        meta("parent", None, None),
        meta("child", Some("parent"), None),
    ];
    let child = metas.iter().find(|entry| entry.id == "child").unwrap();

    assert_eq!(restore_targets(&metas, child).unwrap(), ["parent", "child"]);
}

#[test]
fn delete_family_targets_children_before_parent() {
    let metas = vec![
        meta("root", None, None),
        meta("sub", Some("root"), None),
        meta("nested", Some("sub"), None),
    ];

    assert_eq!(delete_targets(&metas, "root"), ["nested", "sub", "root"]);
}

#[test]
fn delete_single_subagent_targets_only_that_subagent() {
    let metas = vec![meta("root", None, None), meta("sub", Some("root"), None)];

    assert_eq!(delete_targets(&metas, "sub"), ["sub"]);
}
