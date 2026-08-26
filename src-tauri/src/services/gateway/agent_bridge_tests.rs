#[test]
fn gateway_uses_the_shared_stream_owner_and_canonical_admission() {
    let source = [
        include_str!("agent_bridge.rs"),
        include_str!("agent_bridge_run.rs"),
    ]
    .join("\n");

    assert!(source.contains("agent_chat_admission::admit_background"));
    assert!(source.contains("agent_chat_turn::admit_current"));
    assert!(source.contains("agent_chat_streams::finish_active_stream"));
    assert!(!source.contains("conversation_admission::new_turn_for_continuation"));
}
