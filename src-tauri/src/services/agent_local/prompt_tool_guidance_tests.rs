use std::path::Path;

fn prompts() -> [String; 2] {
    [
        super::prompt_compact::build_with_behavior(Path::new("."), false, None, None),
        super::prompt_detailed::build_with_behavior(Path::new("."), false, None, None),
    ]
}

#[test]
fn image_guidance_never_claims_that_metadata_provides_vision() {
    for prompt in prompts() {
        assert!(prompt.contains("read_image. It reports dimensions, format, and file size"));
        assert!(prompt.contains("it does not show visual content"));
        assert!(prompt.contains("resize, crop, or convert images: use process_image"));
        assert!(!prompt.contains("To read/process images"));
    }
}

#[test]
fn discovery_tools_have_distinct_scopes() {
    for prompt in prompts() {
        assert!(prompt.contains("search_mcp_tools for external MCP services"));
        assert!(prompt.contains("search_extension_tools for enabled Beaver plugins"));
        assert!(prompt.contains("load_skill for project instructions and procedures"));
    }
}

#[test]
fn discovery_guidance_disappears_with_its_unavailable_tool() {
    for prompt in prompts() {
        let filtered = super::tool_prompt_filter::filter_system_prompt(
            &prompt,
            &["read_file".to_string()],
        );

        assert!(!filtered.contains("search_mcp_tools for external MCP services"));
        assert!(!filtered.contains("search_extension_tools for enabled Beaver plugins"));
        assert!(!filtered.contains("load_skill for project instructions and procedures"));
    }
}

#[test]
fn bash_guidance_matches_the_optional_timeout_contract() {
    let detailed = super::prompt_detailed::build_with_behavior(Path::new("."), false, None, None);

    assert!(detailed.contains("bash has no forced timeout by default"));
    assert!(detailed.contains("continue it with bash_write"));
    assert!(!detailed.contains("bash times out after 120s"));
}
