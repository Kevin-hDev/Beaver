//! Extraction du résumé final d'un sous-agent depuis ses messages.

use crate::services::agent_local::types_ollama::ChatMessage;

/// Extrait le résumé final du sous-agent depuis l'historique des messages.
///
/// Priorité : dernier message assistant non vide, sinon les derniers
/// résultats d'outils agrégés et tronqués à 2000 caractères.
pub(super) fn extract_summary_from_messages(msgs: &[ChatMessage]) -> String {
    if let Some(m) = msgs
        .iter()
        .rev()
        .find(|m| m.role == "assistant" && !m.content.trim().is_empty())
    {
        return m.content.clone();
    }
    let tool_results: Vec<&str> = msgs
        .iter()
        .rev()
        .take(6)
        .filter(|m| m.role == "tool" && !m.content.trim().is_empty())
        .map(|m| m.content.as_str())
        .collect();
    if !tool_results.is_empty() {
        let joined: String = tool_results
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n---\n");
        let truncated: String = joined.chars().take(2000).collect();
        return format!("[Résultats d'outils]\n{truncated}");
    }
    "Aucune réponse".to_string()
}

#[cfg(test)]
#[path = "subagent_summary_tests.rs"]
mod tests;
