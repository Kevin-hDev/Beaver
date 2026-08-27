use crate::services::agent_local::chat_prompts::*;
use crate::services::agent_local::types_ollama::ChatMessage;

fn make_user_msg(text: &str) -> ChatMessage {
    ChatMessage::user(text.to_string())
}

const TEST_MODEL_SMALL: &str = "gemma-4-e4b";
const TEST_MODEL_LARGE: &str = "qwen3-32b";

#[test]
fn chat_mode_injects_chat_prompt() {
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
        "chat",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("conversational assistant"));
    assert!(sys.content.contains("Beaver"));
    assert!(!sys.content.contains("CL-GO"));
    assert!(sys.content.contains("Chatbot"));
    assert!(!sys.content.contains("autonomous"));
}

#[test]
fn chat_mode_skips_agent_md() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let agent_md = Some("Agent specific rules".to_string());
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        agent_md,
        &[],
        TEST_MODEL_LARGE,
        "chat",
        "",
    );
    let sys = &msgs[0];
    assert!(!sys.content.contains("Agent specific rules"));
}

#[test]
fn chat_mode_skips_skills() {
    let mut msgs = vec![make_user_msg("hello")];
    let wd = std::path::Path::new("/tmp/project");
    let skills = vec![("MySkill".to_string(), "Does stuff".to_string())];
    prepare_messages(
        &mut msgs,
        wd,
        false,
        None,
        true,
        None,
        &skills,
        TEST_MODEL_SMALL,
        "chat",
        "",
    );
    let sys = &msgs[0];
    assert!(!sys.content.contains("MySkill"));
    assert!(!sys.content.contains("Available skills"));
}

#[test]
fn chat_mode_mentions_other_modes() {
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
        "chat",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("Ask for approval"));
    assert!(sys.content.contains("Full access"));
}

#[test]
fn chat_mode_small_model_gets_compact() {
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
        "chat",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("conversational assistant"));
    assert!(!sys.content.contains("Adapt your tone and depth"));
    assert!(sys.content.contains("Never invent files"));
}

#[test]
fn chat_mode_large_model_gets_detailed() {
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
        "chat",
        "",
    );
    let sys = &msgs[0];
    assert!(sys.content.contains("conversational assistant"));
    assert!(sys.content.contains("Adapt your tone and depth"));
    assert!(sys.content.contains("Never invent files"));
}
