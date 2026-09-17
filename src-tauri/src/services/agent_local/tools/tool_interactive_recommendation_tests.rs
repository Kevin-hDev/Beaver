use serde_json::json;

use super::tool_interactive_parse::parse_questions;

#[test]
fn parse_rejects_question_without_recommendation() {
    let err = parse_questions(&json!({
        "questions": [{
            "header": "Plan",
            "question": "Choisir ?",
            "options": [
                {"label": "A", "description": "A"},
                {"label": "B", "description": "B"}
            ]
        }]
    }))
    .unwrap_err();

    assert!(err.contains("exactement"));
}

#[test]
fn parse_rejects_question_with_multiple_recommendations() {
    let err = parse_questions(&json!({
        "questions": [{
            "header": "Plan",
            "question": "Choisir ?",
            "options": [
                {"label": "A", "description": "A", "recommended": true},
                {"label": "B", "description": "B", "recommended": true}
            ]
        }]
    }))
    .unwrap_err();

    assert!(err.contains("exactement"));
}

#[test]
fn parse_accepts_one_recommendation_in_every_question() {
    let questions = parse_questions(&json!({
        "questions": [
            {
                "header": "Portée",
                "question": "Quelle portée ?",
                "options": [
                    {"label": "Locale", "description": "Projet", "recommended": true},
                    {"label": "Globale", "description": "Application"}
                ]
            },
            {
                "header": "Format",
                "question": "Quel format ?",
                "options": [
                    {"label": "Court", "description": "Résumé"},
                    {"label": "Long", "description": "Détaillé", "recommended": true}
                ]
            }
        ]
    }))
    .unwrap();

    assert_eq!(questions.len(), 2);
    assert!(questions[0].options[0].recommended);
    assert!(questions[1].options[1].recommended);
}
