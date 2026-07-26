use serde_json::json;
use std::sync::LazyLock;
use tokio::sync::Mutex;

use super::tool_interactive_parse::{parse_questions, validate_answers};
use super::types_interactive::AgentInteractiveAnswer;

static PENDING_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn valid_args() -> serde_json::Value {
    json!({
        "questions": [{
            "header": "Plan",
            "question": "Quelle suite choisir ?",
            "options": [
                {"id": "fast", "label": "Rapide", "description": "Faire le minimum", "recommended": true},
                {"id": "complete", "label": "Complet", "description": "Faire toute la passe"}
            ]
        }]
    })
}

#[test]
fn parse_accepts_valid_choice_request() {
    let questions = parse_questions(&valid_args()).unwrap();

    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].options.len(), 2);
    assert_eq!(questions[0].options[0].id.as_deref(), Some("fast"));
    assert!(questions[0].options[0].recommended);
}

#[test]
fn parse_rejects_more_than_five_questions() {
    let questions: Vec<_> = (0..6)
        .map(|index| {
            json!({
                "header": format!("Q{index}"),
                "question": "Choisir ?",
                "options": [
                    {"label": "A", "description": "A"},
                    {"label": "B", "description": "B"}
                ]
            })
        })
        .collect();

    assert!(parse_questions(&json!({ "questions": questions })).is_err());
}

#[test]
fn parse_rejects_invalid_option_count() {
    let err = parse_questions(&json!({
        "questions": [{
            "header": "Plan",
            "question": "Choisir ?",
            "options": [{"label": "A", "description": "A"}]
        }]
    }))
    .unwrap_err();

    assert!(err.contains("2 à 4"));
}

#[test]
fn validate_answers_rejects_unknown_label() {
    let questions = parse_questions(&valid_args()).unwrap();
    let err = validate_answers(
        &questions,
        vec![AgentInteractiveAnswer {
            question_index: 0,
            selected_ids: vec![],
            selected_labels: vec!["Inconnu".into()],
            custom_answer: None,
        }],
    )
    .unwrap_err();

    assert!(err.contains("inconnu"));
}

#[test]
fn validate_answers_accepts_known_id() {
    let questions = parse_questions(&valid_args()).unwrap();
    let answers = validate_answers(
        &questions,
        vec![AgentInteractiveAnswer {
            question_index: 0,
            selected_ids: vec!["complete".into()],
            selected_labels: vec!["Complet".into()],
            custom_answer: None,
        }],
    )
    .unwrap();

    assert_eq!(answers[0].selected_ids, vec!["complete"]);
}

#[test]
fn validate_answers_rejects_unknown_id() {
    let questions = parse_questions(&valid_args()).unwrap();
    let err = validate_answers(
        &questions,
        vec![AgentInteractiveAnswer {
            question_index: 0,
            selected_ids: vec!["missing".into()],
            selected_labels: vec!["Complet".into()],
            custom_answer: None,
        }],
    )
    .unwrap_err();

    assert!(err.contains("inconnu"));
}

#[tokio::test]
async fn pending_store_is_bounded_for_tests() {
    let _guard = PENDING_TEST_LOCK.lock().await;
    super::interactive_choice_gate::fill_pending_for_test(64).await;

    assert_eq!(
        super::interactive_choice_gate::pending_len_for_test().await,
        64
    );
    super::interactive_choice_gate::clear_pending_for_test().await;
}

#[tokio::test]
async fn respond_rejects_wrong_session() {
    let _guard = PENDING_TEST_LOCK.lock().await;
    super::interactive_choice_gate::clear_pending_for_test().await;
    super::interactive_choice_gate::insert_pending_for_test("choice-1", "session-a").await;

    let result = super::interactive_choice_gate::respond("session-b", "choice-1", vec![]).await;

    assert!(result.is_err());
    assert_eq!(
        super::interactive_choice_gate::pending_len_for_test().await,
        1
    );
    super::interactive_choice_gate::clear_pending_for_test().await;
}
