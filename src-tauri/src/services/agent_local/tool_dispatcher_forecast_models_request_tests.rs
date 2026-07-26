use super::requested_model_id;

#[test]
fn requested_model_id_is_optional_and_strictly_validated() {
    assert_eq!(
        requested_model_id(&serde_json::json!({})).unwrap(),
        None
    );
    assert!(requested_model_id(&serde_json::json!({
        "requested_model_id": ["moirai-2.0-r-small"]
    }))
    .is_err());
    assert!(requested_model_id(&serde_json::json!({
        "requested_model_id": "../../model"
    }))
    .is_err());
    assert_eq!(
        requested_model_id(&serde_json::json!({
            "requested_model_id": ""
        }))
        .unwrap(),
        None
    );
    assert_eq!(
        requested_model_id(&serde_json::json!({
            "requested_model_id": "moirai-2.0"
        }))
        .unwrap(),
        Some("moirai-2.0")
    );
    assert_eq!(
        requested_model_id(&serde_json::json!({
            "requested_model_id": "moirai-2.0-r-small"
        }))
        .unwrap(),
        Some("moirai-2.0-r-small")
    );
}
