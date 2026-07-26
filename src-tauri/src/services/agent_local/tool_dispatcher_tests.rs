use crate::services::agent_local::tool_dispatcher::enrich_error;
use crate::services::agent_local::types_tools::ToolResult;

// On réexporte les fonctions privées via un module test helper
// en les dupliquant ici pour éviter de les rendre pub dans prod.
// On teste la logique de troncature directement.

const PREVIEW_SIZE: usize = 2_000;

fn max_chars_for_tool_test(name: &str) -> Option<usize> {
    match name {
        "bash" => Some(30_000),
        "grep" => Some(10_000),
        "glob" => Some(5_000),
        "web_fetch" => Some(50_000),
        "web_search" => Some(10_000),
        "list_dir" => Some(10_000),
        _ => None,
    }
}

/// Simule persist_result : crée un fichier dans dir/{uuid}.txt et retourne le chemin.
fn persist_result_test(content: &str, dir: &std::path::Path) -> Option<String> {
    std::fs::create_dir_all(dir).ok()?;
    let file_name = format!("{}.txt", uuid::Uuid::new_v4());
    let path = dir.join(&file_name);
    std::fs::write(&path, content).ok()?;
    Some(path.to_string_lossy().into_owned())
}

fn truncate_result_test(mut result: ToolResult, tool_name: &str) -> ToolResult {
    if result.is_error {
        return result;
    }
    let Some(max) = max_chars_for_tool_test(tool_name) else {
        return result;
    };
    let total = result.content.chars().count();
    if total <= max {
        return result;
    }
    let preview: String = result.content.chars().take(PREVIEW_SIZE).collect();
    let omitted = total - PREVIEW_SIZE;
    let total_kb = total / 1024;
    result.content = format!(
        "[Résultat tronqué — {total_kb} Ko total, preview ci-dessous]\n{preview}\n[{omitted} chars omis]"
    );
    result.truncated = true;
    result
}

fn truncate_result_with_persist_test(
    mut result: ToolResult,
    tool_name: &str,
    persist_dir: &std::path::Path,
) -> ToolResult {
    if result.is_error {
        return result;
    }
    let Some(max) = max_chars_for_tool_test(tool_name) else {
        return result;
    };
    let total = result.content.chars().count();
    if total <= max {
        return result;
    }
    let file_hint = match persist_result_test(&result.content, persist_dir) {
        Some(p) => format!("\n[Résultat complet disponible : {}]", p),
        None => String::new(),
    };
    let preview: String = result.content.chars().take(PREVIEW_SIZE).collect();
    let omitted = total - PREVIEW_SIZE;
    let total_kb = total / 1024;
    result.content = format!(
        "[Résultat tronqué — {total_kb} Ko total, preview ci-dessous]{file_hint}\n{preview}\n[{omitted} chars omis]"
    );
    result.truncated = true;
    result
}

#[test]
fn truncate_under_limit() {
    let content = "a".repeat(100);
    let result = ToolResult::ok(content.clone());
    let out = truncate_result_test(result, "bash");
    assert_eq!(
        out.content, content,
        "Le contenu ne doit pas être modifié sous le seuil"
    );
    assert!(!out.truncated, "truncated doit rester false");
    assert!(!out.is_error);
}

#[test]
fn truncate_over_limit() {
    // bash seuil = 30_000, on crée 31_000 chars
    let content = "x".repeat(31_000);
    let result = ToolResult::ok(content);
    let out = truncate_result_test(result, "bash");
    assert!(out.truncated, "truncated doit être true");
    assert!(!out.is_error);
    // Le message de troncature doit être présent
    assert!(
        out.content.contains("[Résultat tronqué"),
        "Le message de troncature doit être présent"
    );
    assert!(
        out.content.contains("[29000 chars omis]"),
        "Le nombre de chars omis doit être correct (31000 - 2000 = 29000)"
    );
    // Le preview doit faire exactement PREVIEW_SIZE chars (hors header)
    // On vérifie juste que le contenu du preview est bien du 'x'
    assert!(out.content.contains(&"x".repeat(100)));
}

#[test]
fn truncate_error_not_touched() {
    // Les erreurs ne doivent jamais être tronquées même si énormes
    let content = "e".repeat(31_000);
    let result = ToolResult::err(content.clone());
    let out = truncate_result_test(result, "bash");
    assert!(
        !out.truncated,
        "truncated doit rester false pour les erreurs"
    );
    assert!(out.is_error);
    assert_eq!(
        out.content, content,
        "Le contenu d'une erreur ne doit pas être modifié"
    );
}

#[test]
fn truncate_read_file_no_limit() {
    // read_file retourne None dans max_chars_for_tool => pas de troncature
    let content = "r".repeat(100_000);
    let result = ToolResult::ok(content.clone());
    let out = truncate_result_test(result, "read_file");
    assert!(!out.truncated, "read_file ne doit jamais être tronqué");
    assert_eq!(out.content, content);
}

#[test]
fn truncate_utf8_safe() {
    // Vérification que le preview ne coupe pas en milieu de caractère multi-octet
    // On crée un contenu avec des caractères multi-octets au-delà du seuil glob (5_000)
    let emoji = "🎉"; // 4 octets
                      // 5_001 caractères Unicode pour dépasser le seuil glob
    let content: String = emoji.repeat(5_001);
    let total_chars = content.chars().count(); // = 5_001
    let result = ToolResult::ok(content);
    let out = truncate_result_test(result, "glob");
    assert!(out.truncated);
    // Le preview doit être valide UTF-8 (pas de panique)
    assert!(out.content.is_char_boundary(0));
    let omitted = total_chars - PREVIEW_SIZE;
    assert!(
        out.content.contains(&format!("[{omitted} chars omis]")),
        "Le nombre de chars omis doit être correct"
    );
}

#[test]
fn error_no_hint_edit_not_found() {
    let result = ToolResult::err("Chaîne non trouvée dans le fichier".to_string());
    let out = enrich_error(result, "edit_file");
    assert!(out.is_error);
    assert!(
        !out.content.contains("[HINT:"),
        "Pas de hint pour 'non trouvée' — le LLM gère seul"
    );
}

#[test]
fn error_hint_edit_multiple() {
    let result = ToolResult::err("La chaîne apparaît 3 fois dans le fichier".to_string());
    let out = enrich_error(result, "edit_file");
    assert!(out.is_error);
    assert!(
        out.content.contains("[HINT:"),
        "Un hint doit être injecté pour les occurrences multiples"
    );
    assert!(out.content.contains("old_string"));
}

#[test]
fn error_hint_bash_timeout() {
    let result = ToolResult::err("Timeout: commande dépassée".to_string());
    let out = enrich_error(result, "bash");
    assert!(out.is_error);
    assert!(
        out.content.contains("[HINT:"),
        "Un hint doit être injecté pour Timeout"
    );
    assert!(out.content.contains("timeout"));
}

#[test]
fn no_hint_on_success() {
    let result = ToolResult::ok("Tout s'est bien passé".to_string());
    let out = enrich_error(result, "bash");
    assert!(!out.is_error);
    assert!(
        !out.content.contains("[HINT:"),
        "Aucun hint ne doit être ajouté sur un succès"
    );
    assert_eq!(out.content, "Tout s'est bien passé");
}

#[test]
fn truncate_persists_to_disk() {
    // Un résultat bash > 30_000 chars doit être sauvé sur disque
    // et le message de troncature doit contenir le chemin du fichier.
    let content = "y".repeat(31_000);
    let tmp_dir = std::env::temp_dir().join(format!("cl-go-test-{}", uuid::Uuid::new_v4()));
    let result = ToolResult::ok(content.clone());
    let out = truncate_result_with_persist_test(result, "bash", &tmp_dir);

    assert!(out.truncated, "truncated doit être true");
    assert!(!out.is_error);
    assert!(
        out.content.contains("[Résultat tronqué"),
        "Message de troncature attendu"
    );
    assert!(
        out.content.contains("[Résultat complet disponible :"),
        "Le chemin du fichier doit être dans le message"
    );

    // Vérifier qu'un fichier .txt existe dans le dossier temporaire
    let files: Vec<_> = std::fs::read_dir(&tmp_dir)
        .expect("Le dossier de persistence doit exister")
        .flatten()
        .collect();
    assert_eq!(files.len(), 1, "Exactement un fichier doit être créé");
    let saved = std::fs::read_to_string(files[0].path()).expect("Le fichier doit être lisible");
    assert_eq!(saved, content, "Le contenu complet doit être sauvé");

    // Nettoyage
    let _ = std::fs::remove_dir_all(&tmp_dir);
}
