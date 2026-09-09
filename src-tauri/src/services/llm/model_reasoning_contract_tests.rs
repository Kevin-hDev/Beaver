use super::model_reasoning_contract::{
    typescript_bindings, ModelReasoningContract, ReasoningControl,
};
use crate::services::reasoning_continuity::contract::ReasoningModeId;

#[test]
fn vocabulary_serialization_and_lookup_stay_aligned() {
    let expected = [
        (ReasoningModeId::Off, "off"),
        (ReasoningModeId::Auto, "auto"),
        (ReasoningModeId::Minimal, "minimal"),
        (ReasoningModeId::Low, "low"),
        (ReasoningModeId::Medium, "medium"),
        (ReasoningModeId::High, "high"),
        (ReasoningModeId::Xhigh, "xhigh"),
        (ReasoningModeId::Max, "max"),
        (ReasoningModeId::Ultra, "ultra"),
    ];

    for (mode, name) in expected {
        assert_eq!(ReasoningModeId::from_name(Some(name)), Some(mode));
        assert_eq!(serde_json::to_value(mode).unwrap(), name);
    }
}

#[test]
fn checked_in_typescript_matches_the_rust_reasoning_contract() {
    let checked_in =
        include_str!("../../../../src/types/model-reasoning-contract.ts").replace("\r\n", "\n");

    assert_eq!(checked_in, typescript_bindings());
}

#[test]
fn effort_control_serializes_as_a_bounded_tagged_choice() {
    assert_eq!(
        serde_json::to_value(ReasoningControl::Efforts(vec![ReasoningModeId::Minimal])).unwrap(),
        serde_json::json!({"kind":"efforts","efforts":["minimal"]})
    );
}

#[test]
fn unknown_legacy_controls_do_not_invent_a_toggle_or_an_obligation() {
    let contract = ModelReasoningContract::from_legacy_modes(true, &[], None).unwrap();
    assert_eq!(contract.control, ReasoningControl::Unknown);
    assert_eq!(contract.mandatory, None);
    assert_eq!(
        contract.legacy_projection(),
        (vec!["auto".into()], Some("auto".into()))
    );
}

#[test]
fn legacy_projection_preserves_exact_controls_and_defaults() {
    for names in [vec!["off"], vec!["auto", "off"], vec!["low", "high"]] {
        let modes: Vec<String> = names.into_iter().map(str::to_owned).collect();
        let contract = ModelReasoningContract::from_legacy_modes(true, &modes, None).unwrap();
        assert_eq!(contract.legacy_projection(), (modes, None));
    }
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_model_reasoning_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/model-reasoning-contract.ts");

    std::fs::write(path, typescript_bindings()).unwrap();
}
