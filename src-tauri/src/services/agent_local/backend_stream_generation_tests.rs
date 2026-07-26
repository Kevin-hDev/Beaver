#[test]
fn subagent_and_gateway_runs_emit_an_explicit_shared_generation() {
    let subagent = include_str!("subagent_task_stream.rs");
    let gateway = include_str!("../gateway/agent_bridge.rs");

    for (name, source) in [("subagent", subagent), ("gateway", gateway)] {
        assert!(
            source.contains("stream_events::next_generation()"),
            "{name} n'alloue pas de génération explicite"
        );
        assert!(
            source.contains("AgentEventEmitter::with_generation"),
            "{name} émet encore un run anonyme"
        );
    }
}
