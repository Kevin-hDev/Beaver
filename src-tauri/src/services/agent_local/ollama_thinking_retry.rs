//! Détection du bug Ollama "thinking-only" (issue #10976).
//!
//! Quand `think=true` + outils, certains modèles (Qwen3 notamment) expriment
//! leur intention uniquement dans le bloc `<think>` puis s'arrêtent sans
//! répondre ni appeler d'outil. Ollama renvoie alors `done_reason:"stop"` avec
//! `content=""` et `tool_calls=[]`.
//!
//! Ce module expose un détecteur pur (testable) qui identifie ce cas précis
//! afin de relancer la requête avec `think=false` comme contournement.

use crate::services::agent_local::types_ollama::{ChatRequest, OllamaThink};
use crate::services::agent_local::types_stream::StreamResult;

/// Seuil minimum de thinking pour considérer qu'il y a eu vraie réflexion.
/// En dessous, on considère que c'est juste un artefact vide.
const MIN_THINKING_CHARS: usize = 20;

/// Renvoie `true` si le résultat correspond au bug thinking-only :
/// - Le modèle a réfléchi (thinking non vide et significatif)
/// - Mais n'a produit ni contenu ni appel d'outil
/// - Et le stream s'est terminé normalement (`done_reason:"stop"`)
///
/// Dans ce cas, un retry avec `think=false` est la solution de contournement
/// officielle documentée dans l'issue Ollama #10976.
pub fn is_thinking_only_dead_end(result: &StreamResult) -> bool {
    let stopped_clean = result
        .done_reason
        .as_deref()
        .map(|r| r == "stop")
        .unwrap_or(true);

    stopped_clean
        && result.content.trim().is_empty()
        && result.tool_calls.is_empty()
        && result.thinking.trim().chars().count() >= MIN_THINKING_CHARS
}

/// Construit une requête de retry avec `think=false` si le résultat détecté
/// correspond au bug thinking-only. Renvoie `None` sinon (ou si le thinking
/// était déjà désactivé).
pub fn build_thinking_disabled_retry(request: &ChatRequest) -> Option<ChatRequest> {
    if !request.think.as_ref().is_some_and(|t| t.enabled()) {
        return None;
    }
    let mut retry = request.clone();
    retry.think = Some(OllamaThink::Bool(false));
    Some(retry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_stream::StreamResult;

    fn result(content: &str, thinking: &str, tools: usize, done: Option<&str>) -> StreamResult {
        StreamResult {
            content: content.to_string(),
            thinking: thinking.to_string(),
            tool_calls: (0..tools)
                .map(|_| ("x".into(), serde_json::json!({})))
                .collect(),
            done_reason: done.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn detects_thinking_only_dead_end() {
        let r = result(
            "",
            "Je dois créer un fichier puis répondre à l'utilisateur.",
            0,
            Some("stop"),
        );
        assert!(is_thinking_only_dead_end(&r));
    }

    #[test]
    fn ignores_when_content_present() {
        let r = result("Voici la réponse", "réflexion", 0, Some("stop"));
        assert!(!is_thinking_only_dead_end(&r));
    }

    #[test]
    fn ignores_when_tool_call_present() {
        let r = result("", "réflexion", 1, Some("stop"));
        assert!(!is_thinking_only_dead_end(&r));
    }

    #[test]
    fn ignores_when_thinking_too_short() {
        let r = result("", "court", 0, Some("stop"));
        assert!(!is_thinking_only_dead_end(&r));
    }

    #[test]
    fn ignores_when_done_reason_is_tool_call() {
        let r = result(
            "",
            "réflexion longue et détaillée du modèle",
            0,
            Some("tool_calls"),
        );
        assert!(!is_thinking_only_dead_end(&r));
    }

    #[test]
    fn detects_when_done_reason_missing() {
        // done_reason absent = on assume stop (compat anciennes versions)
        let r = result("", "réflexion longue et détaillée du modèle ici", 0, None);
        assert!(is_thinking_only_dead_end(&r));
    }

    #[test]
    fn builds_retry_with_think_false() {
        let req = ChatRequest {
            model: "qwen3.5".into(),
            messages: vec![],
            stream: true,
            tools: Some(vec![]),
            options: None,
            keep_alive: Some("5m".into()),
            think: Some(OllamaThink::Bool(true)),
        };
        let retry = build_thinking_disabled_retry(&req).unwrap();
        assert_eq!(retry.think, Some(OllamaThink::Bool(false)));
        assert_eq!(retry.model, "qwen3.5");
    }

    #[test]
    fn no_retry_when_think_already_false() {
        let req = ChatRequest {
            model: "qwen3.5".into(),
            messages: vec![],
            stream: true,
            tools: Some(vec![]),
            options: None,
            keep_alive: Some("5m".into()),
            think: Some(OllamaThink::Bool(false)),
        };
        assert!(build_thinking_disabled_retry(&req).is_none());
    }
}
