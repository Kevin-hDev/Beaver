#[test]
fn invalid_history_blocks_before_provider_dispatch() {
    let source = include_str!("agent_bridge.rs");
    let conversion = source
        .find("message_convert::build_chat_messages(&session)")
        .expect("gateway history conversion");
    let dispatch = source
        .find("run_stream_task(StreamTaskParams")
        .expect("provider dispatch");
    let guarded_boundary = &source[conversion..dispatch];

    assert!(conversion < dispatch);
    assert!(guarded_boundary.contains(".map_err(BridgeError::SessionError)?;"));
}
