#[test]
fn subagent_and_gateway_runs_emit_an_explicit_shared_generation() {
    let subagent = include_str!("subagent_task_stream.rs");
    let gateway = [
        include_str!("../gateway/agent_bridge.rs"),
        include_str!("../gateway/agent_bridge_run.rs"),
    ]
    .join("\n");

    assert!(subagent.contains("stream_events::next_generation()"));
    assert!(subagent.contains("AgentEventEmitter::with_generation"));
    assert!(gateway.contains("agent_chat_admission::admit_background"));
    assert!(gateway.contains("AgentEventEmitter::with_generation"));
    assert!(gateway.contains("stream.generation"));
}
