use super::core_api_contract::{
    advertised_capabilities, advertised_capabilities_for, validate_negotiated_capabilities,
};

#[test]
fn core_api_contract_is_single_authority() {
    assert_eq!(
        super::core_api_test_support::THIRD_PARTY_IDENTITIES.len(),
        2
    );
    assert_eq!(super::core_api_test_support::OFFICIAL_IDENTITY, "official");
    assert_eq!(super::core_api_test_support::TURN_IDS.len(), 2);
    let all = advertised_capabilities_for(true);
    assert_eq!(
        all,
        [
            "tools",
            "events",
            "ui",
            "skills",
            "resources",
            "richToolResults",
            "models",
            "memory",
            "automations",
            "subagents",
            "toolInterception"
        ]
    );
    let advertised = advertised_capabilities();
    assert!(validate_negotiated_capabilities(
        &advertised
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>()
    )
    .is_ok());
    for capability in ["automations", "subagents", "toolInterception"] {
        assert_eq!(
            validate_negotiated_capabilities(&[capability.to_string()]).is_ok(),
            !cfg!(target_os = "linux")
        );
    }
    assert!(validate_negotiated_capabilities(&["tools".to_string(), "tools".to_string()]).is_err());
    assert_eq!(super::types::CORE_API_METHODS.len(), 16);
    assert_eq!(super::types::CORE_API_METHODS[0].name, "models.list");
    assert_eq!(super::types::CORE_API_METHODS[15].name, "subagents.cancel");
    assert!(super::types::HOST_TO_CORE_IDEMPOTENCE
        .iter()
        .any(|(method, idempotent)| *method == "sessions.get" && *idempotent));
    assert!(super::types::HOST_TO_CORE_IDEMPOTENCE
        .iter()
        .any(|(method, idempotent)| *method == "mcp.tool.call" && !*idempotent));
}

#[test]
fn linux_advertises_only_maintained_extension_capabilities() {
    let capabilities = advertised_capabilities_for(false);
    for capability in [
        "models",
        "memory",
        "automations",
        "subagents",
        "toolInterception",
    ] {
        assert!(!capabilities.contains(&capability));
    }
    for capability in [
        "tools",
        "events",
        "ui",
        "skills",
        "resources",
        "richToolResults",
    ] {
        assert!(capabilities.contains(&capability));
    }

    if cfg!(target_os = "linux") {
        assert_eq!(advertised_capabilities(), capabilities);
    }
}
