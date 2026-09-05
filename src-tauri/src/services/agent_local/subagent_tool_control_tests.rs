use super::subagent_tool_control::is_control_only;
use serde_json::json;

fn call(name: &str) -> (String, serde_json::Value) {
    (name.to_string(), json!({}))
}

#[test]
fn all_subagent_controls_form_a_control_only_batch() {
    let calls = [
        call("list_subagents"),
        call("get_subagent"),
        call("message_subagent"),
        call("cancel_subagent"),
        call("archive_subagent"),
    ];

    assert!(is_control_only(&calls));
}

#[test]
fn delegate_and_mixed_batches_are_not_control_only() {
    assert!(!is_control_only(&[call("delegate_task")]));
    assert!(!is_control_only(&[
        call("list_subagents"),
        call("read_file"),
    ]));
    assert!(!is_control_only(&[]));
}

#[test]
fn api_and_ollama_wait_after_control_batches_before_finishing_tools() {
    for (source, classifier_marker, tool_marker) in [
        (
            include_str!("../llm/agent_loop_tools.rs"),
            "prepare_tool_batch",
            "execute_tool_batch",
        ),
        (
            include_str!("agent_loop_tool_turn.rs"),
            "agent_loop_tool_batch::prepare",
            "agent_loop_tool_batch::execute",
        ),
    ] {
        let classifier = source
            .find(classifier_marker)
            .expect("control batch is classified before tool execution");
        let tools = source.find(tool_marker).expect("tool execution");
        let wait = source
            .find(".wait_after_tool_batch(")
            .expect("shared control wait");
        let after_tools = source
            .find(".finish_tools(")
            .expect("post-tool compression");
        let pre_wait = &source[classifier..wait];

        assert!(classifier < tools);
        assert!(tools < wait);
        assert!(wait < after_tools);
        assert!(!pre_wait.contains(".finish_tools("));
    }
}

#[test]
fn runtime_never_invokes_message_or_cancel_automatically() {
    for source in [
        include_str!("../llm/agent_loop.rs"),
        include_str!("agent_loop.rs"),
        include_str!("agent_loop_tool_turn.rs"),
        include_str!("subagent_orchestration.rs"),
    ] {
        assert!(!source.contains("message_subagent"));
        assert!(!source.contains("cancel_subagent"));
    }
}
