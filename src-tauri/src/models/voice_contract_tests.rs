use super::voice_contract::typescript_bindings;

#[test]
fn an_idle_error_does_not_claim_active_work() {
    let projection = super::voice_contract::VoiceProjection::idle_error(
        crate::services::voice::errors::VoiceError::invalid_settings(),
    );
    assert_eq!(
        projection.phase,
        crate::services::voice::types::VoicePhase::Idle
    );
    assert!(projection.error.is_some());
}

#[test]
fn checked_in_voice_types_match_rust() {
    let checked_in = include_str!("../../../src/types/voice.generated.ts").replace("\r\n", "\n");
    assert_eq!(checked_in, typescript_bindings());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_voice_contract() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/types/voice.generated.ts");
    std::fs::write(path, typescript_bindings()).expect("write voice contract");
}
