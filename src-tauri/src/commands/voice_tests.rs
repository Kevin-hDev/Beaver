use crate::services::voice::contracts::{VoiceAction, VoiceDestination};

#[test]
fn command_action_contract_uses_explicit_variants() {
    let valid = serde_json::json!({
        "action": "start",
        "destination": { "kind": "draft", "draft_key": "draft-1" },
        "context_generation": 1,
        "language": null
    });
    assert!(serde_json::from_value::<VoiceAction>(valid).is_ok());
    let invalid = serde_json::json!({ "action": "unknown", "destination": VoiceDestination::Draft { draft_key: "draft-1".into() } });
    assert!(serde_json::from_value::<VoiceAction>(invalid).is_err());
}
