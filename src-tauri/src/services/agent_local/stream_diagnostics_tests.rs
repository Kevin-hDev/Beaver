use super::stream_diagnostics::push_failure;
use super::stream_diagnostics_failure::classify_error;
use super::types_session::AgentSession;
use chrono::Utc;

#[test]
fn stable_provider_failures_keep_only_actionable_codes() {
    assert_eq!(
        classify_error("provider_request_rejected", false),
        "provider_error"
    );
    assert_eq!(
        classify_error("provider_configuration_invalid", false),
        "provider_configuration_invalid"
    );
}

#[test]
fn provider_overload_is_classified_explicitly() {
    assert_eq!(
        classify_error(
            "Codex: Our servers are currently overloaded. Please try again later.",
            false
        ),
        "provider_overloaded"
    );
}

#[test]
fn stream_failures_are_bounded_and_sanitized() {
    let mut session = test_session();
    for i in 0..25 {
        push_failure(
            &mut session,
            &format!("secret /Users/kevinh/project token-{i}"),
            false,
        );
    }

    assert_eq!(session.stream_failures.len(), 20);
    assert!(session
        .stream_failures
        .iter()
        .all(|failure| failure.code == "stream_error"));
}

fn test_session() -> AgentSession {
    AgentSession {
        schema_version: super::session_limits::CURRENT_SESSION_SCHEMA_VERSION,
        id: "abc-123".into(),
        name: "Test".into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        pinned_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        fast_mode_enabled: false,
        reasoning_mode: None,
        preserve_reasoning: Default::default(),
        accumulated_tokens: 0,
        context_tokens: None,
        compression_profile_selection: None,
        compression_count: 0,
        automatic_compression_guard: Default::default(),
        messages: vec![],
        todos: vec![],
        todo_neglect_count: 0,
        todo_runs: vec![],
        active_todo_run_id: None,
        stream_failures: vec![],
        diagnostic_runs: vec![],
        plan_mode_enabled: false,
        plan_runs: vec![],
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        working_dir_managed: false,
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
        subagent_description: None,
        subagent_color_key: None,
        subagent_summary: None,
        subagent_last_activity: None,
        subagent_queued_prompts: Vec::new(),
        subagent_hidden_reports: Vec::new(),
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
        clone_root_session_id: None,
        git_branch: None,
    }
}

#[test]
fn extension_counts_precede_bounded_display_truncation() {
    let plugin_ids = (0..12)
        .map(|index| format!("plugin.item{index}"))
        .collect::<Vec<_>>();
    let tool_names = (0..12)
        .map(|index| format!("plugin_tool_{index}"))
        .collect::<Vec<_>>();
    let diagnostic = super::extension_tool_diagnostic::structured(
        &super::stream_diagnostics::ExtensionToolDiagnostic {
            origin: super::stream_diagnostics::ExtensionDiagnosticOrigin::Selected,
            reason: super::stream_diagnostics::ExtensionDiagnosticReason::ProviderCapacity,
            correlation_id: None,
            plugin_ids: &plugin_ids,
            tool_names: &tool_names,
            provider_id: "ollama",
            alias_context: &[],
            outcomes: &[],
            additional_tool_count: 5,
            added_tool_count: 0,
        },
    );

    assert_eq!(diagnostic.plugin_count, 12);
    assert_eq!(diagnostic.tool_count, 17);
    assert_eq!(diagnostic.plugin_ids.split(',').count(), 8);
    assert_eq!(diagnostic.canonical_tool_names.split(',').count(), 8);
}

#[test]
fn extension_diagnostic_identifiers_are_never_cut_mid_value() {
    let plugin_ids = ["a", "b", "c"]
        .map(|prefix| format!("{prefix}{}", "x".repeat(95)))
        .to_vec();
    let diagnostic = super::extension_tool_diagnostic::structured(
        &super::stream_diagnostics::ExtensionToolDiagnostic {
            origin: super::stream_diagnostics::ExtensionDiagnosticOrigin::Selected,
            reason: super::stream_diagnostics::ExtensionDiagnosticReason::Protected,
            correlation_id: None,
            plugin_ids: &plugin_ids,
            tool_names: &[],
            provider_id: "ollama",
            alias_context: &[],
            outcomes: &[],
            additional_tool_count: 0,
            added_tool_count: 0,
        },
    );

    assert_eq!(diagnostic.plugin_ids, plugin_ids[..2].join(","));
}

#[tokio::test]
async fn a_started_request_accepts_exactly_one_terminal_transition() {
    let session =
        super::session_store::create_full("Terminal once", "qwen3.5:4b", "ollama", false, None)
            .await
            .unwrap();
    let request_id = super::stream_diagnostics::start_request(&session.id, 1).await;
    super::stream_diagnostics::record_cancelled(&session.id, &request_id).await;
    super::stream_diagnostics::record_failure(
        &session.id,
        Some(&request_id),
        "conversation_admission_failed",
        false,
    )
    .await;

    let stored = super::session_store::get(&session.id).await.unwrap();
    let run = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .unwrap();
    assert_eq!(run.status, "cancelled");
    assert_eq!(
        run.events
            .iter()
            .filter(|event| event.phase == "failed")
            .count(),
        1
    );
    assert!(stored.stream_failures.is_empty());
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn extension_diagnostics_keep_only_bounded_canonical_metadata() {
    let session =
        super::session_store::create_full("Extension diagnostics", "qwen-max", "qwen", false, None)
            .await
            .unwrap();
    let request_id = super::stream_diagnostics::start_request(&session.id, 1).await;
    let sentinel = "SENTINEL user request must never be persisted".to_string();
    super::stream_diagnostics::record_extension_tools(
        &session.id,
        &request_id,
        super::stream_diagnostics::ExtensionToolDiagnostic {
            origin: super::stream_diagnostics::ExtensionDiagnosticOrigin::Selected,
            reason: super::stream_diagnostics::ExtensionDiagnosticReason::Protected,
            correlation_id: None,
            plugin_ids: &["beaver.office.documents".to_string(), sentinel.clone()],
            tool_names: &["search".to_string(), sentinel.clone()],
            provider_id: "qwen",
            alias_context: &[serde_json::json!({"function": {"name": "search"}})],
            outcomes: &[],
            additional_tool_count: 0,
            added_tool_count: 2,
        },
    )
    .await;

    let stored = super::session_store::get(&session.id).await.unwrap();
    let event = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .and_then(|run| run.events.last())
        .unwrap();
    assert_eq!(event.phase, "extension_tools_selected");
    let extension = event.extension.as_ref().unwrap();
    assert_eq!(extension.origin, "extension_tools_selected");
    assert_eq!(extension.reason, "protected");
    assert_eq!(extension.plugin_ids, "beaver.office.documents");
    assert_eq!(extension.canonical_tool_names, "search");
    assert_eq!(extension.provider_aliases, "beaver_search");
    assert_eq!(extension.tool_delta, 2);
    assert!(!serde_json::to_string(event).unwrap().contains("SENTINEL"));
    super::session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn one_refresh_keeps_every_parallel_search_correlation() {
    let session = super::session_store::create_full(
        "Parallel discovery diagnostics",
        "qwen-max",
        "qwen",
        false,
        None,
    )
    .await
    .unwrap();
    let request_id = super::stream_diagnostics::start_request(&session.id, 1).await;
    let mut expected = Vec::new();
    for index in 1..=10_u128 {
        let correlation_id = uuid::Uuid::from_u128(index).to_string();
        expected.push(correlation_id.clone());
        super::stream_diagnostics::record_tool(
            &session.id,
            &request_id,
            crate::services::extensions::SEARCH_TOOL_NAME,
            "started",
            None,
            None,
        )
        .await;
        super::stream_diagnostics::record_extension_tools(
            &session.id,
            &request_id,
            super::stream_diagnostics::ExtensionToolDiagnostic {
                origin: super::stream_diagnostics::ExtensionDiagnosticOrigin::Search,
                reason: super::stream_diagnostics::ExtensionDiagnosticReason::DiscoveryResult,
                correlation_id: Some(&correlation_id),
                plugin_ids: &[],
                tool_names: &[],
                provider_id: "qwen",
                alias_context: &[],
                outcomes: &[],
                additional_tool_count: 0,
                added_tool_count: index as usize,
            },
        )
        .await;
        let result = super::types_tools::ToolResult::ok("bounded result");
        super::stream_diagnostics::record_tool(
            &session.id,
            &request_id,
            crate::services::extensions::SEARCH_TOOL_NAME,
            "completed",
            None,
            Some(&result),
        )
        .await;
    }

    let pending =
        super::stream_diagnostics::pending_extension_searches(&session.id, &request_id).await;
    let added_names = vec!["search".to_string()];
    let alias_context = vec![serde_json::json!({"function": {"name": "search"}})];
    super::stream_diagnostics::record_extension_refreshes(
        &session.id,
        &request_id,
        pending,
        &added_names,
        "qwen",
        &alias_context,
    )
    .await;

    let stored = super::session_store::get(&session.id).await.unwrap();
    let run = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .unwrap();
    let searches = run
        .events
        .iter()
        .filter_map(|event| event.extension.as_ref())
        .filter(|diagnostic| diagnostic.origin == "extension_tool_search")
        .filter_map(|diagnostic| diagnostic.correlation_id.clone())
        .collect::<Vec<_>>();
    let refreshed = run
        .events
        .iter()
        .filter_map(|event| event.extension.as_ref())
        .find(|diagnostic| diagnostic.origin == "extension_tools_refreshed")
        .unwrap();
    assert_eq!(searches, expected);
    assert_eq!(refreshed.related_search_ids, expected);
    assert_eq!(refreshed.tool_delta, 1);
    assert_eq!(refreshed.canonical_tool_names, "search");
    assert_eq!(refreshed.provider_aliases, "beaver_search");
    super::session_store::delete_one(&session.id).await.unwrap();
}
