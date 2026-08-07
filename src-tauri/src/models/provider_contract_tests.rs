use super::provider_contract::{typescript_bindings, ProviderCatalogEntry, ProviderCategory};
use serde_json::json;

fn entry(category: ProviderCategory) -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        id: "provider-id".to_string(),
        display_name: "Provider".to_string(),
        category,
        signup_url: "https://example.com/signup".to_string(),
        base_url: Some("https://api.example.com".to_string()),
        models_endpoint: None,
    }
}

#[test]
fn provider_contract_serializes_the_frontend_shape() {
    let serialized = serde_json::to_value(entry(ProviderCategory::Forecast)).unwrap();

    assert_eq!(
        serialized,
        json!({
            "id": "provider-id",
            "display_name": "Provider",
            "category": "forecast",
            "signup_url": "https://example.com/signup",
            "base_url": "https://api.example.com"
        })
    );
}

#[test]
fn provider_category_rejects_unknown_wire_values() {
    assert_eq!(
        ProviderCategory::from_wire("scraping"),
        Some(ProviderCategory::Scraping)
    );
    assert_eq!(ProviderCategory::from_wire("unknown"), None);
}

#[test]
fn checked_in_typescript_matches_the_rust_contract() {
    let checked_in = include_str!("../../../src/types/api.ts");

    assert_eq!(checked_in, typescript_bindings());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_provider_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/types/api.ts");

    std::fs::write(path, typescript_bindings()).unwrap();
}
