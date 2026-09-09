use super::model_reasoning_contract::{ModelReasoningContract, ReasoningControl};
use super::openrouter_model_metadata::reasoning;
use crate::services::reasoning_continuity::contract::ReasoningModeId;
use serde_json::json;

#[test]
fn reasoning_metadata_keeps_only_known_unique_efforts_and_compatible_default() {
    let parsed = reasoning(&json!({
        "mandatory": true,
        "supported_efforts": [
            "max", "low", "low", "minimal", "none", "medium", "high", "xhigh", "ultra"
        ],
        "default_effort": "none"
    }));

    assert_eq!(parsed, None);
}

#[test]
fn reasoning_metadata_distinguishes_absence_from_an_explicit_empty_object() {
    assert_eq!(reasoning(&json!(null)), None);
    assert!(matches!(
        reasoning(&json!({})).unwrap().control,
        ReasoningControl::ProviderDefault
    ));
    assert!(matches!(
        reasoning(&json!({"supported_efforts": null}))
            .unwrap()
            .control,
        ReasoningControl::Efforts(_)
    ));
    assert!(matches!(
        reasoning(&json!({"mandatory": false, "supported_efforts": []}))
            .unwrap()
            .control,
        ReasoningControl::Toggle
    ));
}

#[test]
fn reasoning_metadata_accepts_a_default_only_when_it_is_advertised() {
    assert_eq!(
        reasoning(&json!({
            "supported_efforts": ["low", "high"],
            "default_effort": "high"
        })),
        Some(ModelReasoningContract {
            mandatory: None,
            default_enabled: None,
            supports_max_tokens: None,
            default_effort: Some(ReasoningModeId::High),
            control: ReasoningControl::Efforts(vec![ReasoningModeId::Low, ReasoningModeId::High]),
        })
    );
    assert_eq!(
        reasoning(&json!({
            "supported_efforts": ["low", "high"],
            "default_effort": "medium"
        })),
        None
    );
}

#[test]
fn non_mandatory_reasoning_may_keep_the_known_off_effort() {
    assert_eq!(
        reasoning(&json!({
            "mandatory": false,
            "supported_efforts": ["off", "none", "low"]
        })),
        None
    );
    assert_eq!(
        reasoning(
            &json!({"mandatory":false,"supported_efforts":["high","none"],"default_effort":"none"})
        ),
        Some(ModelReasoningContract {
            mandatory: Some(false),
            default_enabled: None,
            supports_max_tokens: None,
            default_effort: Some(ReasoningModeId::Off),
            control: ReasoningControl::Efforts(vec![ReasoningModeId::High, ReasoningModeId::Off]),
        })
    );
}

#[test]
fn unsupported_wire_efforts_are_never_offered_as_working_controls() {
    assert_eq!(
        reasoning(&json!({"supported_efforts":["ultra","minimal","low"],"default_effort":"ultra"})),
        None
    );
}

#[test]
fn mandatory_reasoning_without_effort_keeps_a_native_contract() {
    let contract = reasoning(&json!({
        "mandatory": true,
        "default_enabled": true
    }))
    .unwrap();

    assert_eq!(contract.mandatory, Some(true));
    assert_eq!(contract.control, ReasoningControl::ProviderDefault);
    assert_eq!(contract.default_effort, None);
}

#[test]
fn null_efforts_include_the_complete_gateway_vocabulary() {
    let contract = reasoning(&json!({"supported_efforts": null})).unwrap();
    let ReasoningControl::Efforts(modes) = contract.control else {
        panic!("expected gateway efforts");
    };

    assert!(modes.contains(&ReasoningModeId::Minimal));
}

#[test]
fn an_unknown_effort_rejects_the_reasoning_contract() {
    assert!(reasoning(&json!({"supported_efforts": ["quantum"]})).is_none());
}

#[test]
fn optional_reasoning_keeps_off_even_when_efforts_omit_none() {
    let contract = reasoning(&json!({
        "mandatory": false,
        "supported_efforts": ["low", "high"]
    }))
    .unwrap();

    assert_eq!(
        contract.control,
        ReasoningControl::Efforts(vec![
            ReasoningModeId::Off,
            ReasoningModeId::Low,
            ReasoningModeId::High,
        ])
    );
}

#[test]
fn disabled_default_does_not_erase_the_effort_used_when_enabled() {
    let contract = reasoning(&json!({
        "mandatory": false,
        "default_enabled": false,
        "supported_efforts": ["low", "high"],
        "default_effort": "high"
    }))
    .unwrap();

    assert_eq!(contract.default_effort, Some(ReasoningModeId::High));
    assert_eq!(contract.legacy_projection().1.as_deref(), Some("off"));
}

#[test]
fn budget_only_reasoning_uses_the_provider_default_without_an_effort() {
    let contract = reasoning(&json!({"supports_max_tokens": true})).unwrap();

    assert_eq!(contract.supports_max_tokens, Some(true));
    assert_eq!(contract.control, ReasoningControl::ProviderDefault);
    assert_eq!(contract.default_effort, None);
}

#[test]
fn mandatory_null_efforts_never_offer_disabling_reasoning() {
    let contract = reasoning(&json!({"mandatory":true,"supported_efforts":null})).unwrap();
    let (modes, _) = contract.legacy_projection();
    assert!(!modes.iter().any(|mode| mode == "off"));
    assert!(modes.iter().any(|mode| mode == "minimal"));
}
