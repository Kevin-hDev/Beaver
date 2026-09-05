use std::path::Path;

fn prompts() -> [String; 2] {
    [
        super::prompt_compact::build_with_behavior(Path::new("."), false, None, None),
        super::prompt_detailed::build_with_behavior(Path::new("."), false, None, None),
    ]
}

#[test]
fn image_guidance_uses_the_single_image_tool() {
    for prompt in prompts() {
        assert!(prompt
            .contains("inspect metadata, resize, crop, or convert images: use transform_image"));
        assert!(!prompt.contains("To read/process images"));
    }
}

#[test]
fn discovery_tools_have_distinct_scopes() {
    for prompt in prompts() {
        assert!(prompt.contains("search_mcp_tools for external MCP services"));
        assert!(
            prompt.contains("Inspect directly with inspect_extensions using 1–4 known exact IDs")
        );
        assert!(
            prompt.contains("list_extensions only for a complete view, descriptions, or counts")
        );
        assert!(prompt.contains("Never use lexical or keyword search"));
        assert!(prompt.contains("load_skill for project instructions and procedures"));
    }
}

#[test]
fn discovery_guidance_disappears_with_its_unavailable_tool() {
    for prompt in prompts() {
        let filtered =
            super::tool_prompt_filter::filter_system_prompt(&prompt, &["read_file".to_string()]);

        assert!(!filtered.contains("search_mcp_tools for external MCP services"));
        assert!(!filtered
            .contains("Inspect directly with inspect_extensions using 1–4 known exact IDs"));
        assert!(
            !filtered.contains("list_extensions only for a complete view, descriptions, or counts")
        );
        assert!(!filtered.contains("load_skill for project instructions and procedures"));
    }
}

#[test]
fn bash_guidance_matches_the_optional_timeout_contract() {
    let detailed = super::prompt_detailed::build_with_behavior(Path::new("."), false, None, None);

    assert!(detailed.contains("bash has no forced timeout by default"));
    assert!(detailed.contains("continue it with bash_control"));
    assert!(!detailed.contains("bash times out after 120s"));
}

#[test]
fn agent_tools_hide_backend_permission_modes() {
    let forbidden = [
        "session-allowed",
        "Ask for approval mode",
        "Full access mode",
        "Requires user confirmation",
    ];

    for definition in super::tool_definitions::native_tool_definitions() {
        let name = definition["function"]["name"].as_str().unwrap_or_default();
        let description = definition["function"]["description"]
            .as_str()
            .unwrap_or_default();
        for phrase in forbidden {
            assert!(
                !description.contains(phrase),
                "{name} exposes backend permission detail: {phrase}"
            );
        }
    }
}

#[test]
fn agent_prompts_do_not_claim_an_active_permission_mode() {
    for prompt in prompts() {
        assert!(!prompt.contains("full access to the user's system"));
        assert!(!prompt.contains("session-allowed"));
        assert!(!prompt.contains("Ask for approval mode"));
        assert!(!prompt.contains("Full access mode"));
    }
}

#[test]
fn chatbot_prompts_expose_only_real_web_capabilities() {
    for prompt in [
        super::prompt_chat_compact::build_with_behavior(Path::new("."), None),
        super::prompt_chat_detailed::build_with_behavior(Path::new("."), None),
    ] {
        assert!(prompt.contains("web_search"));
        assert!(prompt.contains("web_fetch"));
        assert!(!prompt.contains("ask_user_choice"));
        assert!(!prompt.contains("Chatbot"));
        assert!(!prompt.contains("Ask for approval"));
        assert!(!prompt.contains("Full access"));
    }
}
