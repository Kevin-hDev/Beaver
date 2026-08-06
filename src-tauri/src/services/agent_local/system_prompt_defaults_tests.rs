use super::system_prompt_defaults::beaver_prompt;
use super::system_prompt_types::{PromptMode, PromptTier};

#[test]
fn every_mode_and_tier_has_its_own_visible_beaver_prompt() {
    let chatbot_compact = beaver_prompt(PromptMode::Chatbot, PromptTier::Compact);
    let chatbot_detailed = beaver_prompt(PromptMode::Chatbot, PromptTier::Detailed);
    let agentic_compact = beaver_prompt(PromptMode::Agentic, PromptTier::Compact);
    let agentic_detailed = beaver_prompt(PromptMode::Agentic, PromptTier::Detailed);

    for prompt in [
        &chatbot_compact,
        &chatbot_detailed,
        &agentic_compact,
        &agentic_detailed,
    ] {
        assert!(!prompt.trim().is_empty());
        assert!(!prompt.contains("# Environment"));
    }
    assert_ne!(chatbot_compact, chatbot_detailed);
    assert_ne!(agentic_compact, agentic_detailed);
    assert_ne!(chatbot_compact, agentic_compact);
}
