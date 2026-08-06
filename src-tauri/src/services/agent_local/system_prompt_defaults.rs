use super::system_prompt_types::{PromptMode, PromptTier};
use std::path::Path;

pub fn beaver_prompt(mode: PromptMode, tier: PromptTier) -> String {
    match (mode, tier) {
        (PromptMode::Chatbot, PromptTier::Compact) => {
            super::prompt_chat_compact::build_with_behavior(Path::new("."), None)
        }
        (PromptMode::Chatbot, PromptTier::Detailed) => {
            super::prompt_chat_detailed::build_with_behavior(Path::new("."), None)
        }
        (PromptMode::Agentic, PromptTier::Compact) => {
            super::prompt_compact::build_with_behavior(Path::new("."), false, None, None)
        }
        (PromptMode::Agentic, PromptTier::Detailed) => {
            super::prompt_detailed::build_with_behavior(Path::new("."), false, None, None)
        }
    }
}

pub fn mode_for_permission(permission_mode: &str) -> PromptMode {
    if permission_mode == "chat" {
        PromptMode::Chatbot
    } else {
        PromptMode::Agentic
    }
}

pub fn tier_for_model(model: &str) -> PromptTier {
    match super::model_size::detect_tier(model) {
        super::model_size::PromptTier::Compact => PromptTier::Compact,
        super::model_size::PromptTier::Detailed => PromptTier::Detailed,
    }
}
