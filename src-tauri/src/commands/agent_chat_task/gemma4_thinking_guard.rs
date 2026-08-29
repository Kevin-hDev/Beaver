use crate::services::agent_local::types_ollama::ChatMessage;

const GEMMA4_THINKING_GUARD: &str = "Règle interne pour Gemma 4 : n'écris jamais ton raisonnement interne dans la réponse normale. Si tu produis du raisonnement, encadre-le strictement avec <think>...</think>, puis écris uniquement la réponse finale hors de ces balises.";

pub(crate) fn apply(messages: &mut Vec<ChatMessage>, provider: &str, model: &str) {
    if !crate::services::llm::route_profile::requires_gemma4_thinking_guard(provider, model) {
        return;
    }
    if messages
        .iter()
        .any(|message| message.content.contains(GEMMA4_THINKING_GUARD))
    {
        return;
    }
    if let Some(system) = messages.iter_mut().find(|message| message.role == "system") {
        system.content.push_str("\n\n");
        system.content.push_str(GEMMA4_THINKING_GUARD);
        return;
    }
    messages.insert(0, ChatMessage::system(GEMMA4_THINKING_GUARD.to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(role: &str, content: &str) -> ChatMessage {
        match role {
            "system" => ChatMessage::system(content.to_string()),
            "user" => ChatMessage::user(content.to_string()),
            "assistant" => ChatMessage::assistant(content.to_string(), None, None, None, None),
            "tool" => ChatMessage::tool(content.to_string(), None, None),
            other => panic!("unsupported chat role in test/setup: {other}"),
        }
    }

    #[test]
    fn guard_is_limited_to_google_and_openrouter() {
        assert!(
            crate::services::llm::route_profile::requires_gemma4_thinking_guard(
                "google",
                "gemma-4-26b"
            )
        );
        assert!(
            crate::services::llm::route_profile::requires_gemma4_thinking_guard(
                "openrouter",
                "google/gemma 4 31b"
            )
        );
        assert!(
            !crate::services::llm::route_profile::requires_gemma4_thinking_guard(
                "google",
                "gemini-2.5-flash"
            )
        );
        assert!(
            !crate::services::llm::route_profile::requires_gemma4_thinking_guard(
                "ollama",
                "gemma4:31b"
            )
        );
    }

    #[test]
    fn guard_is_appended_once_to_system_message() {
        let mut messages = vec![message("system", "Base"), message("user", "Salut")];

        apply(&mut messages, "google", "gemma-4-31b");
        apply(&mut messages, "google", "gemma-4-31b");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(
            messages[0].content.matches(GEMMA4_THINKING_GUARD).count(),
            1
        );
    }
}
