use super::access_log::AccessResult;
use super::call_context::ExtensionCallContext;
use super::host_identity::HostIdentity;
use super::types::ExtensionApiLevel;
use serde_json::Value;

#[test]
fn core_call_log_uses_only_bounded_generic_fields() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("access.jsonl");
    let context = ExtensionCallContext::for_test(
        HostIdentity::ThirdParty("com.example.safe".to_string()),
        ExtensionApiLevel::Stable,
    );
    let sentinels = "secret-MUST-NOT-LEAK https://private.invalid /Users/private/file";

    super::access_log::write_core_at(
        &path,
        &context,
        &format!("unknown.{sentinels}"),
        AccessResult::Denied,
    )
    .unwrap();

    let text = std::fs::read_to_string(path).unwrap();
    assert!(!text.contains("MUST-NOT-LEAK"));
    assert!(!text.contains("https://"));
    assert!(!text.contains("/Users/"));
    let value: Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(value["kind"], "core_call");
    assert_eq!(value["method"], "unknown");
    assert_eq!(value["result"], "denied");
    assert!(value["correlationId"].as_str().unwrap().len() == 36);
    assert!(value.get("params").is_none());
}

#[test]
fn host_started_log_has_generation_and_pid_but_no_method() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("access.jsonl");

    super::access_log::write_host_started_at(
        &path,
        HostIdentity::Official,
        42,
        1234,
        AccessResult::Granted,
    )
    .unwrap();

    let text = std::fs::read_to_string(path).unwrap();
    let value: Value = serde_json::from_str(text.trim()).unwrap();
    assert_eq!(value["kind"], "host_started");
    assert_eq!(value["identity"], "beaver.official");
    assert_eq!(value["generation"], 42);
    assert_eq!(value["pid"], 1234);
    assert!(value.get("method").is_none());
    assert!(value.get("correlationId").is_none());
}
