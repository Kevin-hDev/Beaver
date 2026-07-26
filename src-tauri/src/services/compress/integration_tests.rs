#[cfg(test)]
mod integration {
    use crate::services::agent_local::types_ollama::ChatMessage;
    use crate::services::compress::{engine, prompt, token_estimate};

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    // 1. Flux complet : conversation → estimation → compression → vérification
    #[test]
    fn full_flow_compression() {
        let mut messages = vec![
            msg("system", "You are an assistant"),
            msg("user", &"Long question ".repeat(500)),
            msg("assistant", &"Long answer ".repeat(500)),
            msg("user", "Continue"),
            msg("assistant", &"More ".repeat(500)),
        ];
        let used = token_estimate::estimate_tokens(&messages);
        assert!(used > 0);
        let summary = "The user asked long questions and the assistant answered.";
        let pre_count = engine::apply_compression(&mut messages, summary, true);
        assert!(pre_count > messages.len());
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "You are an assistant");
        assert!(messages.iter().any(|m| m.content.contains(summary)));
    }

    // 2. extract_summary avec tags
    #[test]
    fn prompt_extraction_with_tags() {
        let response =
            "<analysis>Internal thinking</analysis>\n<summary>The real summary</summary>";
        assert_eq!(prompt::extract_summary(response), "The real summary");
    }

    // 3. extract_summary sans tags (fallback)
    #[test]
    fn prompt_extraction_no_tags() {
        let response = "No tags here";
        assert_eq!(prompt::extract_summary(response), "No tags here");
    }

    // 4. Compression préserve le system prompt
    #[test]
    fn compression_preserves_system_prompt() {
        let mut messages = vec![
            msg("system", "You are Claude"),
            msg("user", "Hello"),
            msg("assistant", "Hi"),
        ];
        engine::apply_compression(&mut messages, "Summary", false);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[0].content, "You are Claude");
    }

    // 5. La requête de compression exclut le system prompt
    #[test]
    fn compression_request_excludes_system() {
        let messages = vec![msg("system", "Secret system prompt"), msg("user", "Hello")];
        let request = engine::build_compression_request_content(&messages, None);
        assert!(request.iter().all(|m| m.content != "Secret system prompt"));
    }

    // 6. should_auto_compress intègre toutes les conditions
    #[test]
    fn auto_compress_full_check() {
        // Tout activé, modèle éligible, au-dessus du seuil
        assert!(engine::should_auto_compress(
            true, 131_072, 100_000, 86_000, 85
        ));
        // Compression désactivée
        assert!(!engine::should_auto_compress(
            false, 131_072, 100_000, 86_000, 85
        ));
        // Petit contexte : le réglage utilisateur prime, donc compression possible.
        assert!(engine::should_auto_compress(
            true, 32_768, 32_768, 30_000, 85
        ));
        // Contexte inconnu : on ne compresse pas à l'aveugle.
        assert!(!engine::should_auto_compress(true, 0, 0, 30_000, 85));
        // Sous le seuil
        assert!(!engine::should_auto_compress(
            true, 131_072, 100_000, 80_000, 85
        ));
    }

    // 7. Compression sans system prompt initial
    #[test]
    fn compression_without_system_prompt() {
        let mut messages = vec![msg("user", "Hello"), msg("assistant", "Hi")];
        engine::apply_compression(&mut messages, "Summary", false);
        // Pas de system original → boundary (system) + summary (user)
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert!(messages[0].content.contains("Compression boundary"));
    }

    // 8. Le prompt de compression contient les 9 sections
    #[test]
    fn compression_prompt_completeness() {
        let p = prompt::build_compression_prompt(None);
        assert!(p.contains("Primary Request"));
        assert!(p.contains("Key Technical"));
        assert!(p.contains("Files and Code"));
        assert!(p.contains("Errors and Fixes"));
        assert!(p.contains("Problem Solving"));
        assert!(p.contains("User Messages"));
        assert!(p.contains("Pending Tasks"));
        assert!(p.contains("Current Work"));
        assert!(p.contains("Next Step"));
    }

    // 9. format_summary_message avec auto-compression
    #[test]
    fn summary_message_auto_mode() {
        let m = prompt::format_summary_message("Test", true);
        assert!(m.contains("without asking"));
        assert!(m.contains("Resume directly"));
    }

    // 10. format_summary_message mode manuel
    #[test]
    fn summary_message_manual_mode() {
        let m = prompt::format_summary_message("Test", false);
        assert!(!m.contains("without asking"));
        assert!(m.contains("previous conversation"));
    }
}
