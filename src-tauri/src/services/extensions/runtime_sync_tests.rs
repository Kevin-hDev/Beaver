use super::protocol::HostExtensionSpec;
use super::runtime_sync::accepts_contributions;
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionEffect, ExtensionManifest, ExtensionTool,
};
use serde_json::json;

#[test]
fn missing_or_invalid_tool_effect_is_revalidated_as_unknown() {
    for value in [
        json!({"name":"tool","description":"Tool","parameters":{}}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":"root"}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":null}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":42}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":[]}),
    ] {
        let tool: ExtensionTool = serde_json::from_value(value).unwrap();
        assert_eq!(tool.effect, ExtensionEffect::Unknown);
    }
}

#[test]
fn stable_extensions_cannot_replace_core_tools() {
    let spec = HostExtensionSpec {
        id: "com.example.stable".to_string(),
        main_path: "/tmp/index.ts".to_string(),
        manifest: ExtensionManifest {
            id: "com.example.stable".to_string(),
            name: "Stable".to_string(),
            version: "1.0.0".to_string(),
            beaver_api: "1".to_string(),
            runtime: "node".to_string(),
            main: Some("index.ts".to_string()),
            ui: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
    };
    let contributions = ExtensionContributions {
        tools: vec![ExtensionTool {
            name: "web_search".to_string(),
            description: "Replacement".to_string(),
            parameters: json!({"type": "object"}),
            effect: ExtensionEffect::Unknown,
            replaces_core: true,
        }],
        events: Vec::new(),
    };

    assert!(!accepts_contributions(&spec, &contributions));
}
