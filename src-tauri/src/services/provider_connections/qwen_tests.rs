use super::qwen::{
    decode, encode, resolve_qwen_endpoint, validate_qwen_connection, QwenConnectionInput,
    QwenEndpointMode, QwenRegion,
};

fn input(region: QwenRegion, endpoint_mode: QwenEndpointMode) -> QwenConnectionInput {
    QwenConnectionInput {
        region,
        endpoint_mode,
        workspace_id: None,
    }
}

#[test]
fn qwen_endpoint_is_resolved_from_the_closed_region_table() {
    let endpoint = resolve_qwen_endpoint(&input(QwenRegion::Singapore, QwenEndpointMode::Shared))
        .expect("valid endpoint");
    assert_eq!(
        endpoint.base_url,
        "https://dashscope-intl.aliyuncs.com/compatible-mode/v1"
    );
    assert_eq!(endpoint.models_url, format!("{}/models", endpoint.base_url));
}

#[test]
fn qwen_workspace_requires_a_bounded_workspace_id() {
    assert!(
        validate_qwen_connection(&input(QwenRegion::Singapore, QwenEndpointMode::Workspace))
            .is_err()
    );
    for workspace in ["", "-team", "team-", "TEAM", "../team"] {
        let mut candidate = input(QwenRegion::Singapore, QwenEndpointMode::Workspace);
        candidate.workspace_id = Some(workspace.to_string());
        assert!(validate_qwen_connection(&candidate).is_err(), "{workspace}");
    }
}

#[test]
fn region_and_mode_matrix_is_closed() {
    assert!(resolve_qwen_endpoint(&input(QwenRegion::Tokyo, QwenEndpointMode::Shared)).is_err());
    assert!(resolve_qwen_endpoint(&input(QwenRegion::Frankfurt, QwenEndpointMode::Trial)).is_err());
    assert!(resolve_qwen_endpoint(&input(QwenRegion::HongKong, QwenEndpointMode::Trial)).is_ok());
}

#[test]
fn connection_record_round_trips_only_current_valid_records() {
    let value = input(QwenRegion::Virginia, QwenEndpointMode::Shared);
    let encoded = encode(value.clone()).unwrap();
    assert_eq!(decode(&encoded).unwrap().connection, value);
    assert!(decode(r#"{"schema_version":2}"#).is_err());
}
