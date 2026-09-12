use super::*;

#[test]
fn generated_output_uses_cumulative_text_units() {
    let mut result = StreamResult::default();

    assert_eq!(result.record_generated_text("abc"), 1);
    assert_eq!(result.record_generated_text("de"), 2);
    assert_eq!(result.estimated_output_tokens(), 2);
}

#[test]
fn generated_output_includes_tool_call_payload() {
    let mut result = StreamResult::default();
    result.record_generated_tool_call("bash", &serde_json::json!({ "command": "pwd" }));

    assert!(result.estimated_output_tokens() > 0);
}

#[test]
fn generated_output_saturates_at_u32_max() {
    assert_eq!(bounded_tokens(usize::MAX), u32::MAX);
}

#[test]
fn provider_input_and_output_replace_only_their_matching_estimates() {
    let mut result = StreamResult {
        prompt_tokens: Some(100),
        usage: Some(crate::services::provider_usage::RequestUsage {
            input_tokens: Some(100),
            output_tokens: Some(50),
            ..Default::default()
        }),
        ..Default::default()
    };
    result.record_generated_text("estimated output");

    let (input, output) = resolved_result_counts(&result, ContextCountSource::Provider);

    assert_eq!(input, Some((100, ContextCountSource::Provider)));
    assert_eq!(output, Some((50, ContextCountSource::Provider)));
}

#[test]
fn absent_provider_input_keeps_it_absent_but_can_keep_native_output() {
    let result = StreamResult {
        eval_count: Some(12),
        ..Default::default()
    };

    assert_eq!(
        resolved_result_counts(&result, ContextCountSource::NativeCounter).0,
        None
    );
    assert_eq!(
        resolved_result_counts(&result, ContextCountSource::NativeCounter).1,
        Some((12, ContextCountSource::NativeCounter))
    );
}

#[tokio::test]
async fn provider_measurement_and_output_follow_the_active_request_identity() {
    let session = super::super::session_store::create_full(
        "Context runtime",
        "gpt-5",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    let make_journal = || {
        super::super::conversation_journal::ConversationJournal::new(
            session.id.clone(),
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(),
        )
        .unwrap()
    };
    let emitter = super::super::stream_events::AgentEventEmitter::test(session.id.clone());
    let first = make_journal();
    first.activate_context_request().await.unwrap();
    ContextAttempt {
        on_event: &emitter,
        journal: Some(&first),
        provider_id: "openai",
        model: "gpt-5",
        turn: 0,
        attempt: 1,
        context_limit: 200_000,
        measured_input_source: ContextCountSource::Provider,
    }
    .persist_preparation(120, Default::default())
    .await
    .unwrap();
    ContextAttempt {
        on_event: &emitter,
        journal: Some(&first),
        provider_id: "openai",
        model: "gpt-5",
        turn: 0,
        attempt: 1,
        context_limit: 200_000,
        measured_input_source: ContextCountSource::Provider,
    }
    .persist_result(&StreamResult {
            prompt_tokens: Some(100),
            usage: Some(crate::services::provider_usage::RequestUsage {
                input_tokens: Some(100),
                output_tokens: Some(50),
                ..Default::default()
            }),
            ..Default::default()
        })
    .await
    .unwrap();

    let second = make_journal();
    second.activate_context_request().await.unwrap();
    ContextAttempt {
        on_event: &emitter,
        journal: Some(&second),
        provider_id: "openai",
        model: "gpt-5",
        turn: 0,
        attempt: 1,
        context_limit: 100_000,
        measured_input_source: ContextCountSource::Provider,
    }
    .persist_preparation(80, Default::default())
    .await
    .unwrap();
    ContextAttempt {
        on_event: &emitter,
        journal: Some(&second),
        provider_id: "openai",
        model: "gpt-5",
        turn: 0,
        attempt: 1,
        context_limit: 100_000,
        measured_input_source: ContextCountSource::Provider,
    }
    .persist_result(&StreamResult {
            usage: Some(crate::services::provider_usage::RequestUsage {
                output_tokens: Some(20),
                ..Default::default()
            }),
            ..Default::default()
        })
    .await
    .unwrap();

    let saved = super::super::session_store::get(&session.id).await.unwrap();
    let current = saved.context_usage.current_preparation.unwrap();
    assert_eq!(current.input.tokens, Some(80));
    assert_eq!(current.state, ContextPreparationState::Completed);
    let measurement = saved.context_usage.last_measurement.unwrap();
    assert_eq!(measurement.input.tokens, Some(100));
    assert_eq!(measurement.context_limit, Some(200_000));
    assert_eq!(saved.context_usage.last_output.unwrap().output.tokens, Some(20));
    super::super::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn prepared_payload_persists_only_its_provider_overhead_delta() {
    let session = super::super::session_store::create_full(
        "Provider overhead",
        "gpt-5",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    let journal = super::super::conversation_journal::ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .unwrap();
    journal.activate_context_request().await.unwrap();
    let emitter = super::super::stream_events::AgentEventEmitter::test(session.id.clone());
    let count = |tokens| ContextTokenCount {
        tokens: Some(tokens),
        capacity_tokens: Some(tokens),
        source: Some(ContextCountSource::Heuristic),
        coverage: ContextCountCoverage::Complete,
    };
    let attempt = PreparedContextAttempt::new(
        ContextAttempt {
            on_event: &emitter,
            journal: Some(&journal),
            provider_id: "openai",
            model: "gpt-5",
            turn: 0,
            attempt: 1,
            context_limit: 200_000,
            measured_input_source: ContextCountSource::Provider,
        },
        Default::default(),
    )
    .with_baseline_count(count(100));

    attempt.persist_payload(count(137)).await.unwrap();

    let saved = super::super::session_store::get(&session.id).await.unwrap();
    assert_eq!(
        saved
            .context_usage
            .current_preparation
            .unwrap()
            .transient_overhead_tokens,
        37
    );
    super::super::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
