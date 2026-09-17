use super::tool_path_args::{fields, PathUse};
use std::collections::BTreeSet;

#[test]
fn metadata_covers_every_native_path_argument() {
    for definition in super::tool_definitions::native_tool_definitions() {
        let name = definition["function"]["name"].as_str().unwrap();
        let schema_fields = definition["function"]["parameters"]["properties"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(name, _)| name.as_str())
            .filter(|name| {
                matches!(
                    *name,
                    "path" | "input_path" | "output_path" | "file_path" | "workdir"
                )
            })
            .collect::<BTreeSet<_>>();
        let metadata_fields = fields(name)
            .iter()
            .map(|field| field.name)
            .collect::<BTreeSet<_>>();

        assert_eq!(metadata_fields, schema_fields, "path metadata for {name}");
    }
}

#[test]
fn image_input_and_output_keep_distinct_roles() {
    let image = fields("transform_image");
    assert_eq!(image.len(), 2);
    assert_eq!(image[0].name, "input_path");
    assert!(image[0].usage == PathUse::Read);
    assert_eq!(image[1].name, "output_path");
    assert!(image[1].usage == PathUse::Write);
}
