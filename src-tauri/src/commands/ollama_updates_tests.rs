use super::OllamaModelUpdate;

#[test]
fn model_update_ipc_identifies_the_exact_remote_revision() {
    let value = serde_json::to_value(OllamaModelUpdate {
        full_name: "llama3:latest".into(),
        family: "llama3".into(),
        tag: "latest".into(),
        latest_digest: "abc123".into(),
    })
    .unwrap();

    assert_eq!(value["latestDigest"], "abc123");
}
