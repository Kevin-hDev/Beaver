use super::*;

#[test]
fn report_distinguishes_loaded_and_omitted_plugins() {
    let output = render(vec![
        DiscoveryLine {
            plugin_name: "Documents".to_string(),
            status: DiscoveryStatus::Loaded,
        },
        DiscoveryLine {
            plugin_name: "Large".to_string(),
            status: DiscoveryStatus::ProviderLimit,
        },
    ]);

    assert!(output.contains("Documents : outils chargés"));
    assert!(output.contains("Large : non chargé"));
}

#[test]
fn discovered_ids_are_unique_and_bounded() {
    let mut ids = vec!["example.one".to_string()];
    push_unique(&mut ids, "example.one");
    push_unique(&mut ids, "example.two");

    assert_eq!(ids, vec!["example.one", "example.two"]);
}

#[test]
fn a_plugin_without_tools_is_never_reported_as_available() {
    assert_eq!(
        existing_status(0, true),
        Some(DiscoveryStatus::NoTools)
    );
}
