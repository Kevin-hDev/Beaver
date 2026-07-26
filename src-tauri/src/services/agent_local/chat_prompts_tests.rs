use crate::services::agent_local::chat_prompts::*;
use crate::services::agent_local::types_ollama::ChatMessage;

fn make_user_msg(text: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: text.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    }
}

const TEST_MODEL_SMALL: &str = "gemma-4-e4b";
const TEST_MODEL_LARGE: &str = "qwen3-32b";

// === prepend_agent_md_context tests ===

#[test]
fn agent_md_prepended_before_system() {
    let mut msgs = vec![make_user_msg("hello")];
    prepend_agent_md_context(&mut msgs, Some("Be helpful and concise.".to_string()));
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].role, "system");
    assert!(msgs[0].content.contains("Be helpful and concise."));
}

#[test]
fn agent_md_appended_to_existing_system() {
    let mut msgs = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "Existing system prompt".to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        },
        make_user_msg("hello"),
    ];
    prepend_agent_md_context(&mut msgs, Some("Agent rules".to_string()));
    assert_eq!(msgs.len(), 2);
    assert!(msgs[0].content.contains("Existing system prompt"));
    assert!(msgs[0].content.contains("Agent rules"));
}

#[test]
fn agent_md_none_does_nothing() {
    let mut msgs = vec![make_user_msg("hello")];
    prepend_agent_md_context(&mut msgs, None);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].role, "user");
}

// === prepare_messages tests (agent modes) ===

#[test]
fn prepare_tool_capable_injects_agent_md() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let agent_md = Some("Tu réponds en français.".to_string());
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        agent_md,
        &[],
        TEST_MODEL_SMALL,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert_eq!(sys.role, "system");
    assert!(sys.content.contains("Tu réponds en français."));
    assert!(sys.content.contains("/tmp/project"));
}

#[test]
fn prepare_tool_capable_injects_tool_prompt() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let agent_md = Some("Use JSON output.".to_string());
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        agent_md,
        &[],
        TEST_MODEL_LARGE,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("Use JSON output."));
    assert!(sys.content.contains("autonomous"));
}

#[test]
fn prepare_tool_capable_injects_work_updates_guidance() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &[],
        TEST_MODEL_LARGE,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("<communication_during_work>"));
    assert!(sys
        .content
        .contains("Normal assistant text is visible to the user"));
    assert!(sys.content.contains("meaningful milestones"));
    assert!(sys.content.contains("</communication_during_work>"));
}

#[test]
fn prepare_not_tool_capable_no_agent_md() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        false,
        None,
        &[],
        TEST_MODEL_SMALL,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("/tmp/project"));
}

#[test]
fn prepare_tool_capable_no_agent_md_file() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &[],
        TEST_MODEL_SMALL,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("autonomous"));
    assert!(sys.content.contains("/tmp/project"));
}

#[test]
fn prepare_existing_system_prompt_preserved() {
    let mut msgs = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "Custom system prompt from frontend".to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        },
        make_user_msg("hello"),
    ];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        Some("Agent rules".to_string()),
        &[],
        TEST_MODEL_LARGE,
        "auto",
        "",
    );
    assert_eq!(msgs.len(), 2);
    let sys = &msgs[0];
    assert!(sys.content.contains("Custom system prompt from frontend"));
    assert!(sys.content.contains("Agent rules"));
}

// === skills listing + model tier tests ===

#[test]
fn prepare_with_skills_injects_listing() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills = vec![
        (
            "Test Greeting".to_string(),
            "Force une salutation".to_string(),
        ),
        ("Debug Helper".to_string(), "Aide au debug".to_string()),
    ];
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &skills,
        TEST_MODEL_SMALL,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("Test Greeting"));
    assert!(sys.content.contains("Force une salutation"));
    assert!(sys.content.contains("load_skill"));
}

#[test]
fn prepare_without_tools_no_skills() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills = vec![(
        "Test Greeting".to_string(),
        "Force une salutation".to_string(),
    )];
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        false,
        None,
        &skills,
        TEST_MODEL_SMALL,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(!sys.content.contains("Test Greeting"));
}

#[test]
fn prepare_empty_skills_no_section() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &[],
        TEST_MODEL_SMALL,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(!sys.content.contains("Available skills"));
}

#[test]
fn small_model_gets_compact_prompt() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &[],
        TEST_MODEL_SMALL,
        "auto",
        "",
    );
    let sys = &msgs[0];
    // Compact now includes git guidance, but stays distinguishable from detailed.
    assert!(!sys.content.contains("highly capable"));
    assert!(!sys.content.contains("# Error handling"));
    assert!(sys.content.contains("autonomous"));
    // New rules must be present even in the compact variant.
    assert!(sys.content.contains("Never claim a task is complete"));
    assert!(sys.content.contains("Advance on your own"));
    assert!(sys.content.contains("verify it actually works"));
}

#[test]
fn large_model_gets_detailed_prompt() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &[],
        TEST_MODEL_LARGE,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("Working with git"));
    assert!(sys.content.contains("Error handling"));
    assert!(sys.content.contains("highly capable"));
    // Detailed-only rules.
    assert!(sys.content.contains("# Verification"));
    assert!(sys.content.contains("# Honesty"));
    assert!(sys.content.contains("does NOT extend to the next context"));
    assert!(sys.content.contains("prompt injection"));
    assert!(sys.content.contains("file_path:line_number"));
}

// === new: env_section format tests ===

#[test]
fn env_section_no_boundary_disclaimer() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &[],
        TEST_MODEL_LARGE,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(
        !sys.content.contains("not a boundary"),
        "Le prompt ne doit plus contenir 'not a boundary'"
    );
    assert!(
        !sys.content.contains("anywhere on the system"),
        "Le prompt ne doit plus contenir 'anywhere on the system'"
    );
}

#[test]
fn env_section_has_shell_and_os() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    prepare_messages(
        &mut msgs,
        wd,
        true,
        None,
        true,
        None,
        &[],
        TEST_MODEL_LARGE,
        "auto",
        "",
    );
    let sys = &msgs[0];
    assert!(
        sys.content.contains("Shell:"),
        "Le prompt doit contenir 'Shell:'"
    );
    assert!(
        sys.content.contains("OS Version:"),
        "Le prompt doit contenir 'OS Version:'"
    );
    assert!(sys.content.contains("Is a git repository: true"));
}
