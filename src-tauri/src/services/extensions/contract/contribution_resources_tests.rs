use super::types::{ExtensionResource, ExtensionResourceType, MAX_EXTENSION_TEXT_CHARS};

fn resource() -> ExtensionResource {
    ExtensionResource {
        id: "reference".to_string(),
        name: "Reference".to_string(),
        description: "Description.".to_string(),
        resource_type: ExtensionResourceType::Text,
        path: "resources/reference.txt".to_string(),
    }
}

#[test]
fn resources_reject_duplicate_ids_and_overlong_human_metadata() {
    let resource = resource();
    assert!(
        super::contribution_resources::validate(&[resource.clone(), resource.clone()]).is_err()
    );

    let mut overlong = resource;
    overlong.description = "🦫".repeat(MAX_EXTENSION_TEXT_CHARS + 1);
    assert!(super::contribution_resources::validate(&[overlong]).is_err());
}

#[test]
fn resources_accept_normal_relative_paths() {
    assert!(super::contribution_resources::validate(&[resource()]).is_ok());
}

#[test]
fn resources_enforce_the_generated_collection_limit() {
    let resources = (0..=super::types::MAX_RESOURCES_PER_EXTENSION)
        .map(|index| ExtensionResource {
            id: format!("resource-{index}"),
            ..resource()
        })
        .collect::<Vec<_>>();

    assert!(super::contribution_resources::validate(&resources).is_err());
}
