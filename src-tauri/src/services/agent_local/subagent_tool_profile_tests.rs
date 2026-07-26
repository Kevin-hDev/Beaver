use super::subagent_tool_profile::SubagentToolProfile;

#[test]
fn explorer_profile_has_exact_capabilities() {
    assert_eq!(
        SubagentToolProfile::Explorer.tool_names(false),
        vec![
            "bash",
            "read_file",
            "list_dir",
            "grep",
            "glob",
            "web_search",
            "web_fetch",
        ]
    );
}

#[test]
fn coder_profile_only_adds_load_skill_when_enabled() {
    let expected = vec![
        "bash",
        "read_file",
        "write_file",
        "edit_file",
        "list_dir",
        "grep",
        "glob",
        "web_search",
        "web_fetch",
    ];
    assert_eq!(SubagentToolProfile::Coder.tool_names(false), expected);
    let mut with_skill = expected;
    with_skill.push("load_skill");
    assert_eq!(SubagentToolProfile::Coder.tool_names(true), with_skill);
}

#[test]
fn definitions_and_prompt_names_match_executable_names() {
    for (profile, skills_enabled) in [
        (SubagentToolProfile::Explorer, false),
        (SubagentToolProfile::Coder, false),
        (SubagentToolProfile::Coder, true),
    ] {
        let executable = profile.tool_names(skills_enabled);
        let definitions = profile.definition_names(skills_enabled);
        let described = profile.prompt_tool_names(skills_enabled);
        assert_eq!(definitions, executable);
        assert_eq!(described, executable);
    }
}

#[test]
fn invalid_or_nested_profiles_fail_closed() {
    assert_eq!(SubagentToolProfile::from_session_type(Some("explorer")), Ok(SubagentToolProfile::Explorer));
    assert_eq!(SubagentToolProfile::from_session_type(Some("coder")), Ok(SubagentToolProfile::Coder));
    assert!(SubagentToolProfile::from_session_type(None).is_err());
    assert!(SubagentToolProfile::from_session_type(Some("unknown")).is_err());
    assert!(!SubagentToolProfile::Coder.allows("delegate_task", true));
    assert!(!SubagentToolProfile::Explorer.allows("load_skill", true));
}
