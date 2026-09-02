use super::*;
use serde_json::json;

#[tokio::test]
async fn rejects_an_extension_identity_claimed_by_node_params() {
    let result = call(
        &super::super::host_identity::HostIdentity::ThirdParty("com.example.bound".to_string()),
        &super::super::types::ExtensionApiLevel::Stable,
        "app.info",
        Some(&json!({"extensionId": "com.beaver.office.word"})),
    )
    .await;

    assert!(result.is_err());
}
