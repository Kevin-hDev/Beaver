use super::protocol::HostExtensionSpec;
use super::runtime_sync::accepts_contributions;
use super::types::{ExtensionApiLevel, ExtensionContributions, ExtensionManifest, ExtensionTool};
use serde_json::json;

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
            replaces_core: true,
        }],
        events: Vec::new(),
    };

    assert!(!accepts_contributions(&spec, &contributions));
}
