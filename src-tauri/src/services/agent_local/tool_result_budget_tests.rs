//! Tests séparés pour `tool_result_budget`.
//! Couvre les cas requis : under budget, truncate oldest, preserve last two.

#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_result_budget::{apply_budget, CLEARED_PLACEHOLDER};
    use crate::services::agent_local::types_ollama::ChatMessage;

    fn tool_msg(content: &str) -> ChatMessage {
        ChatMessage {
            role: "tool".to_string(),
            content: content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    fn assistant_msg(content: &str) -> ChatMessage {
        ChatMessage {
            role: "assistant".to_string(),
            content: content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Messages dont le total est sous le budget ne doivent pas être modifiés.
    #[test]
    fn under_budget_no_change() {
        let mut msgs = vec![tool_msg("résultat court"), tool_msg("autre résultat court")];
        let original_0 = msgs[0].content.clone();
        let original_1 = msgs[1].content.clone();

        apply_budget(&mut msgs);

        assert_eq!(msgs[0].content, original_0);
        assert_eq!(msgs[1].content, original_1);
    }

    /// Quand le total dépasse le budget, les plus anciens sont tronqués en premier.
    #[test]
    fn over_budget_truncates_oldest() {
        let big = "a".repeat(60_000); // 60k chars chacun → 3 × 60k = 180k > 100k
        let mut msgs = vec![
            tool_msg(&big), // index 0 — le plus ancien → tronqué
            tool_msg(&big), // index 1 — préservé (2 derniers)
            tool_msg(&big), // index 2 — préservé (2 derniers)
        ];

        apply_budget(&mut msgs);

        assert_eq!(
            msgs[0].content, CLEARED_PLACEHOLDER,
            "Le plus ancien doit être tronqué"
        );
        assert_eq!(msgs[1].content, big, "L'avant-dernier doit être préservé");
        assert_eq!(msgs[2].content, big, "Le dernier doit être préservé");
    }

    /// Les 2 derniers tool results sont toujours préservés même si le budget est dépassé.
    #[test]
    fn keeps_last_two_tool_results() {
        let big = "b".repeat(40_000); // 4 × 40k = 160k > 100k
        let mut msgs = vec![
            tool_msg(&big), // index 0 → candidat à la troncature
            tool_msg(&big), // index 1 → candidat à la troncature
            tool_msg(&big), // index 2 → préservé (2 derniers)
            tool_msg(&big), // index 3 → préservé (2 derniers)
        ];

        apply_budget(&mut msgs);

        // Les 2 derniers intacts
        assert_eq!(msgs[2].content, big, "msgs[2] doit être préservé");
        assert_eq!(msgs[3].content, big, "msgs[3] doit être préservé");
    }

    /// Les messages non-tool ne sont pas comptés ni modifiés.
    #[test]
    fn non_tool_messages_not_touched() {
        let big = "c".repeat(60_000);
        let mut msgs = vec![
            assistant_msg(&big), // pas un tool result
            tool_msg(&big),      // index 1 — préservé (2 derniers)
            tool_msg(&big),      // index 2 — préservé (2 derniers)
        ];

        apply_budget(&mut msgs);

        // assistant message jamais touché
        assert_eq!(msgs[0].content, big);
        // 2 tool results : les 2 derniers → préservés
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }
}
