use super::*;

#[test]
fn classifies_plain_turn_as_final() {
    let result = StreamResult {
        content_chunks: vec!["done".into()],
        ..Default::default()
    };
    assert!(matches!(
        content_phase_for_result(&result, false, false),
        Some(TokenPhase::Final)
    ));
}

#[test]
fn classifies_tool_turn_as_work() {
    let result = StreamResult {
        content_chunks: vec!["working".into()],
        tool_calls: vec![("bash".into(), serde_json::json!({}))],
        ..Default::default()
    };
    assert!(matches!(
        content_phase_for_result(&result, false, false),
        Some(TokenPhase::Work)
    ));
}

#[test]
fn forces_plain_turn_as_work_while_subagent_runs() {
    let result = StreamResult {
        content_chunks: vec!["still working".into()],
        ..Default::default()
    };
    assert!(matches!(
        content_phase_for_result(&result, false, true),
        Some(TokenPhase::Work)
    ));
}

#[test]
fn hides_plan_mode_tool_content() {
    let result = StreamResult {
        content_chunks: vec!["hidden".into()],
        tool_calls: vec![("write_plan".into(), serde_json::json!({}))],
        ..Default::default()
    };
    assert!(content_phase_for_result(&result, true, false).is_none());
}

#[test]
fn token_phase_serializes_when_present() {
    let event = StreamEvent::Token {
        content: "answer".into(),
        token_count: 1,
        tps: 0.0,
        phase: Some(TokenPhase::Final),
    };
    let value = serde_json::to_value(event).expect("serialize token");
    assert_eq!(value["data"]["phase"], "final");
}

#[test]
fn content_phase_serializes_when_present() {
    let event = StreamEvent::ContentPhase {
        phase: TokenPhase::Work,
    };
    let value = serde_json::to_value(event).expect("serialize content phase");
    assert_eq!(value["event"], "contentPhase");
    assert_eq!(value["data"]["phase"], "work");
}

#[test]
fn classifies_interrupted_text_as_work() {
    let result = StreamResult {
        content_chunks: vec!["partial".into()],
        ..Default::default()
    };
    assert!(matches!(
        interrupted_phase_for_result(&result),
        Some(TokenPhase::Work)
    ));
}

#[test]
fn ignores_empty_interrupted_text() {
    assert!(interrupted_phase_for_result(&StreamResult::default()).is_none());
}
