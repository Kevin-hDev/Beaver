use super::*;

#[test]
fn grok_and_kimi_oauth_use_the_native_agent_loop() {
    assert_eq!(chat_engine("xai-oauth"), ChatEngine::NativeApi);
    assert_eq!(chat_engine("moonshot-oauth"), ChatEngine::NativeApi);
    assert_eq!(chat_engine("xai"), ChatEngine::NativeApi);
    assert_eq!(chat_engine("moonshot"), ChatEngine::NativeApi);
}

#[test]
fn mascot_outcome_covers_every_terminal_path() {
    assert_eq!(
        recovery::mascot_outcome(&Ok(CompletedStreamTurn::compression(Vec::new()))),
        crate::services::mascot::MascotSessionOutcome::Success
    );
    assert_eq!(
        recovery::mascot_outcome(&Err("Annulé".into())),
        crate::services::mascot::MascotSessionOutcome::Cancelled
    );
    assert_eq!(
        recovery::mascot_outcome(&Err("indisponible".into())),
        crate::services::mascot::MascotSessionOutcome::Failed
    );
    assert_eq!(
        context_lifecycle::terminal_state(&Ok(CompletedStreamTurn::compression(Vec::new()))),
        crate::services::agent_local::context_usage_record::ContextPreparationState::Completed
    );
    assert_eq!(
        context_lifecycle::terminal_state(&Err("Annulé".into())),
        crate::services::agent_local::context_usage_record::ContextPreparationState::Interrupted
    );
    assert_eq!(
        context_lifecycle::terminal_state(&Err("indisponible".into())),
        crate::services::agent_local::context_usage_record::ContextPreparationState::Failed
    );
}

#[test]
fn every_stream_consumer_receives_a_boxed_agent_loop() {
    type StreamRun = fn(StreamTaskParams) -> SpawnedStreamTask;

    let _run: StreamRun = run_stream_task;
}
