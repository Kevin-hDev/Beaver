//! Conversion des messages `role: "tool"` pour le wire-format Ollama.
//!
//! Certains modèles (Gemma, et d'autres via Ollama) ne reconnaissent pas le
//! rôle `role: "tool"` du standard OpenAI : leur chat template le traite comme
//! un nouveau tour utilisateur, ce qui pousse le modèle à émettre un token de
//! fin immédiatement après (arrêt après 1 token).
//!
//! La solution documentée par Google pour Gemma consiste à renvoyer le
//! résultat d'un outil comme un message `role: "user"` encapsulé dans des
//! balises `<tool_response>`. On applique cette transformation uniquement au
//! payload HTTP envoyé à Ollama — le `Vec<ChatMessage>` interne conserve
//! toujours `role: "tool"` pour le budget, la compression et les diagnostics.

use super::types_ollama::ChatMessage;

/// Wrap d'un contenu de résultat d'outil.
pub(crate) const TOOL_RESPONSE_OPEN: &str = "<tool_response>";
pub(crate) const TOOL_RESPONSE_CLOSE: &str = "</tool_response>";

/// Clone `messages` et transforme chaque entrée `role: "tool"` en `role: "user"`
/// avec le contenu encapsulé dans des balises `<tool_response>`.
///
/// Les autres messages (user/assistant/system) sont clonés à l'identique.
/// Le Vec source n'est jamais muté.
pub fn wrap_tool_results(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    messages
        .iter()
        .map(|m| {
            if m.role != "tool" {
                return m.clone();
            }
            let wrapped = match &m.tool_name {
                Some(name) if !name.is_empty() => {
                    format!(
                        "{TOOL_RESPONSE_OPEN} name=\"{name}\"\n{}\n{TOOL_RESPONSE_CLOSE}",
                        m.content
                    )
                }
                _ => format!("{TOOL_RESPONSE_OPEN}\n{}\n{TOOL_RESPONSE_CLOSE}", m.content),
            };
            ChatMessage {
                role: "user".to_string(),
                content: wrapped,
                // tool_calls / tool_name / tool_call_id : sans objet pour un message user
                images: None,
                tool_calls: None,
                tool_name: None,
                tool_call_id: None,
                reasoning_content: None,
            }
        })
        .collect()
}
