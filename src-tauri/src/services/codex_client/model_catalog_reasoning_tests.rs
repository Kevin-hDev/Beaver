use super::*;
use crate::services::codex_client::request_build::build_codex_request;
use crate::services::llm::fast_mode::FastModeRequest;

fn catalog(target: serde_json::Value, modes: &[&str]) -> Vec<CatalogModel> {
    let value = serde_json::json!({"models": [{
        "slug": "gpt-5.6-sol", "display_name": "Sol", "context_window": 272000,
        "supported_reasoning_levels": modes.iter().map(|effort| serde_json::json!({"effort": effort})).collect::<Vec<_>>(),
        "multi_agent_reasoning_effort": target
    }]});
    super::super::parse_response(serde_json::from_value(value).unwrap()).unwrap()
}

#[test]
fn ultra_translates_catalog_target_without_changing_other_request_fields() {
    for (target, modes, expected) in [
        (
            serde_json::json!("high"),
            vec!["high", "max", "ultra"],
            "high",
        ),
        (serde_json::Value::Null, vec!["high", "max", "ultra"], "max"),
        (
            serde_json::json!("ultra"),
            vec!["high", "max", "ultra"],
            "max",
        ),
        (
            serde_json::json!("invalid"),
            vec!["low", "high", "ultra"],
            "high",
        ),
    ] {
        let mut request = build_codex_request(
            "gpt-5.6-sol",
            &[],
            &[],
            Some("ultra"),
            Some("test-session"),
            FastModeRequest::Fast,
        );
        let before = serde_json::to_value(&request).unwrap();
        apply_ultra(&mut request, &catalog(target, &modes)).unwrap();
        let after = serde_json::to_value(request).unwrap();
        let mut expected_body = before;
        expected_body["reasoning"]["effort"] = expected.into();
        assert_eq!(after, expected_body);
    }
}

#[test]
fn ultra_rejects_missing_model_or_absent_encodable_effort() {
    for models in [
        Vec::new(),
        catalog(serde_json::Value::Null, &["ultra"]),
        catalog(serde_json::json!("max"), &["max"]),
    ] {
        let mut request = build_codex_request(
            "gpt-5.6-sol",
            &[],
            &[],
            Some("ultra"),
            None,
            FastModeRequest::Standard,
        );
        assert!(apply_ultra(&mut request, &models).is_err());
    }
}

#[tokio::test]
async fn ordinary_efforts_do_not_fetch_catalog_or_change_payload() {
    for mode in ["low", "medium", "high", "xhigh", "max"] {
        let mut request = build_codex_request(
            "gpt-5.6-sol",
            &[],
            &[],
            Some(mode),
            None,
            FastModeRequest::Standard,
        );
        let before = serde_json::to_value(&request).unwrap();
        prepare(&mut request).await.unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), before);
    }
}
