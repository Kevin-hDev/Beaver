use super::host_identity::HostIdentity;
use super::protocol::HostExtensionSpec;
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionEffect, ExtensionManifest, ExtensionSkill,
    ExtensionTool,
};
use serde_json::json;

fn specification() -> HostExtensionSpec {
    HostExtensionSpec {
        id: "com.example.skills".to_string(),
        main_path: "/untrusted/index.mjs".to_string(),
        manifest: ExtensionManifest {
            id: "com.example.skills".to_string(),
            name: "Skills".to_string(),
            version: "1.0.0".to_string(),
            beaver_api: "1".to_string(),
            runtime: "node".to_string(),
            main: Some("index.mjs".to_string()),
            ui: None,
            ui_legacy: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
    }
}

#[test]
fn compromised_host_contribution_is_refused_before_runtime_projection() {
    let specification = specification();
    let contributions = ExtensionContributions {
        skills: vec![ExtensionSkill {
            id: "guide".to_string(),
            name: "Guide".to_string(),
            description: "Description.".to_string(),
            path: "../outside.md".to_string(),
        }],
        ..Default::default()
    };

    assert!(matches!(super::runtime_sync_contributions::validate(
        &HostIdentity::ThirdParty(specification.id.clone()),
        &specification.id,
        &specification,
        contributions,
    ), Err(super::runtime_sync_contributions::ValidationError::InvalidContribution)));
}

#[test]
fn stable_core_replacement_is_classified_as_advanced_required() {
    let specification = specification();
    let contributions = ExtensionContributions {
        tools: vec![ExtensionTool {
            name: "web_search".to_string(),
            description: "Replacement".to_string(),
            parameters: json!({"type": "object"}),
            effect: ExtensionEffect::Unknown,
            replaces_core: true,
        }],
        ..Default::default()
    };

    assert!(matches!(super::runtime_sync_contributions::validate(
        &HostIdentity::ThirdParty(specification.id.clone()),
        &specification.id,
        &specification,
        contributions,
    ), Err(super::runtime_sync_contributions::ValidationError::AdvancedRequired)));
}
