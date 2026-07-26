use crate::services::agent_local::types_ollama::ChatMessage;

const MAX_TOTAL_RESULT_CHARS: usize = 100_000;

/// Supprime les dossiers de résultats persistés datant de plus de 24h.
pub fn cleanup_old_results() {
    let persist_dir = crate::services::paths::data_dir().join("tool-results");
    if let Ok(entries) = std::fs::read_dir(&persist_dir) {
        let cutoff = std::time::SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(86400))
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        }
    }
}
pub const CLEARED_PLACEHOLDER: &str =
    "[Résultat précédent tronqué — relancer le tool si nécessaire]";

const PERSIST_MARKER: &str = "[Résultat complet disponible : ";

/// Extrait le chemin du fichier persistant s'il est présent dans le contenu.
fn extract_persist_path(content: &str) -> Option<&str> {
    let start = content.find(PERSIST_MARKER)? + PERSIST_MARKER.len();
    let end = content[start..].find(']')? + start;
    Some(&content[start..end])
}

/// Vérifie si un message a déjà été tronqué (commence par CLEARED_PLACEHOLDER).
fn is_cleared(content: &str) -> bool {
    content.starts_with(CLEARED_PLACEHOLDER)
}

/// Applique un budget sur les anciens tool results.
///
/// Si la somme totale des chars des messages `role="tool"` dépasse
/// `MAX_TOTAL_RESULT_CHARS`, les plus anciens sont remplacés par
/// `CLEARED_PLACEHOLDER` jusqu'à redescendre sous le budget.
/// Les 2 derniers tool results sont toujours préservés.
/// Les messages déjà tronqués (commençant par CLEARED_PLACEHOLDER) ne comptent pas.
pub fn apply_budget(messages: &mut [ChatMessage]) {
    // 1. Collecter les indices des messages role="tool" non déjà tronqués
    let tool_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == "tool" && !is_cleared(&m.content))
        .map(|(i, _)| i)
        .collect();

    // 2. Calculer le total des chars (Unicode — pas des octets)
    let total: usize = tool_indices
        .iter()
        .map(|&i| messages[i].content.chars().count())
        .sum();

    // 3. Si sous le budget, rien à faire
    if total <= MAX_TOTAL_RESULT_CHARS {
        return;
    }

    // 4. Tronquer les plus anciens en gardant les 2 derniers intacts
    let preserve_from = tool_indices.len().saturating_sub(2);
    let candidates = &tool_indices[..preserve_from];

    let mut remaining = total;
    for &idx in candidates {
        if remaining <= MAX_TOTAL_RESULT_CHARS {
            break;
        }
        remaining -= messages[idx].content.chars().count();
        let persist_info = extract_persist_path(&messages[idx].content)
            .map(|p| format!(" Fichier complet : {p}"))
            .unwrap_or_default();
        messages[idx].content = format!("{CLEARED_PLACEHOLDER}{persist_info}");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

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

    fn user_msg(content: &str) -> ChatMessage {
        ChatMessage {
            role: "user".to_string(),
            content: content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    #[test]
    fn under_budget_no_change() {
        let mut msgs = vec![tool_msg("small"), tool_msg("also small")];
        apply_budget(&mut msgs);
        assert_eq!(msgs[0].content, "small");
        assert_eq!(msgs[1].content, "also small");
    }

    #[test]
    fn over_budget_truncates_oldest() {
        // 3 tool results : 60k + 60k + 60k = 180k > 100k
        let big = "x".repeat(60_000);
        let mut msgs = vec![tool_msg(&big), tool_msg(&big), tool_msg(&big)];
        apply_budget(&mut msgs);
        // Le 1er (oldest) doit être tronqué
        assert_eq!(msgs[0].content, CLEARED_PLACEHOLDER);
        // Les 2 derniers préservés
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }

    #[test]
    fn keeps_last_two_tool_results() {
        // 4 tool results de 40k chacun = 160k > 100k
        let big = "y".repeat(40_000);
        let mut msgs = vec![
            tool_msg(&big),
            tool_msg(&big),
            tool_msg(&big),
            tool_msg(&big),
        ];
        apply_budget(&mut msgs);
        // Les 2 derniers doivent être préservés
        assert_eq!(msgs[2].content, big);
        assert_eq!(msgs[3].content, big);
    }

    #[test]
    fn non_tool_messages_ignored() {
        let big = "z".repeat(60_000);
        let mut msgs = vec![user_msg(&big), tool_msg(&big), tool_msg(&big)];
        apply_budget(&mut msgs);
        // user message non touché
        assert_eq!(msgs[0].content, big);
        // 2 tool results sous 100k ensemble (60k+60k=120k > 100k mais 2 derniers préservés)
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }

    #[test]
    fn preserves_persist_path_when_clearing() {
        // Si le contenu contient un lien vers le fichier persistant, il doit être conservé
        let path = "/tmp/cl-go/tool-results/abc123/result.txt";
        let big_with_link = format!(
            "{}\n[Résultat complet disponible : {path}]",
            "x".repeat(60_000)
        );
        let big = "x".repeat(60_000);
        let mut msgs = vec![tool_msg(&big_with_link), tool_msg(&big), tool_msg(&big)];
        apply_budget(&mut msgs);
        // Le 1er doit être tronqué mais conserver le lien
        let expected = format!("{CLEARED_PLACEHOLDER} Fichier complet : {path}");
        assert_eq!(
            msgs[0].content, expected,
            "le lien persistant doit être conservé"
        );
        // Les 2 derniers préservés
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }

    #[test]
    fn already_cleared_not_recounted() {
        let big = "w".repeat(60_000);
        let mut msgs = vec![
            ChatMessage {
                role: "tool".to_string(),
                content: CLEARED_PLACEHOLDER.to_string(),
                images: None,
                tool_calls: None,
                tool_name: None,
                tool_call_id: None,
                reasoning_content: None,
            },
            tool_msg(&big),
            tool_msg(&big),
        ];
        // total actif = 120k > 100k, mais seulement 2 tool results actifs
        // → les 2 derniers sont préservés, rien n'est tronqué
        apply_budget(&mut msgs);
        assert_eq!(msgs[0].content, CLEARED_PLACEHOLDER);
        assert_eq!(msgs[1].content, big);
        assert_eq!(msgs[2].content, big);
    }
}
