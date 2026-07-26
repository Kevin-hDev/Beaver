#[test]
fn spawned_event_is_not_published_during_preparation() {
    let prepare = include_str!("tool_delegate.rs");
    let dispatch = include_str!("tool_dispatcher_delegate.rs");

    assert!(!prepare.contains("parent_emitter.send(StreamEvent::SubagentSpawned"));
    assert!(dispatch.contains("subagent_spawn_channel::send"));
}
