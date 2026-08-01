use super::*;

#[test]
fn report_distinguishes_loaded_and_omitted_plugins() {
    let result = discovery_result(vec![
        DiscoveryLine {
            plugin_name: "Documents".to_string(),
            status: DiscoveryStatus::Loaded,
        },
        DiscoveryLine {
            plugin_name: "Large".to_string(),
            status: DiscoveryStatus::ProviderLimit,
        },
        DiscoveryLine {
            plugin_name: "Overflow".to_string(),
            status: DiscoveryStatus::DiscoveryLimit,
        },
        DiscoveryLine {
            plugin_name: "Unavailable".to_string(),
            status: DiscoveryStatus::Unavailable,
        },
    ]);

    let output = &result.content;
    assert!(output.contains("Documents : outils chargés"));
    assert!(output.contains("Large : non chargé"));
    assert!(output.contains("limite de plugins découverts"));
    assert!(output.contains("outils indisponibles dans cette requête"));
    assert_eq!(
        result.status,
        super::super::tool_result_contract::ToolResultStatus::Partial
    );
    assert_eq!(result.warnings.len(), 1);
}

#[test]
fn fully_loaded_discovery_is_a_clean_success() {
    let result = discovery_result(vec![DiscoveryLine {
        plugin_name: "Documents".to_string(),
        status: DiscoveryStatus::Loaded,
    }]);

    assert_eq!(
        result.status,
        super::super::tool_result_contract::ToolResultStatus::Success
    );
    assert!(result.warnings.is_empty());
}

#[test]
fn discovered_ids_are_unique_and_bounded() {
    let mut ids = vec!["example.one".to_string()];
    assert!(push_unique(&mut ids, "example.one"));
    assert!(push_unique(&mut ids, "example.two"));

    assert_eq!(ids, vec!["example.one", "example.two"]);
}

#[test]
fn discovery_limit_is_reported_separately() {
    let mut ids = (0..crate::services::extensions::MAX_DISCOVERED_PLUGINS)
        .map(|index| format!("example.plugin{index}"))
        .collect::<Vec<_>>();

    assert!(!push_unique(&mut ids, "example.overflow"));
}

#[test]
fn a_plugin_without_tools_is_never_reported_as_available() {
    assert_eq!(
        existing_status(0, true),
        Some(DiscoveryStatus::NoTools)
    );
}
