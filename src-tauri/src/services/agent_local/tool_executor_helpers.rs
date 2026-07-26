pub use super::tool_executor_results::{emit_tool_result, push_tool_message, push_tool_result};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use tokio_util::sync::CancellationToken;

pub fn check_write_guard(
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    write_guard: &mut WriteGuard,
) -> Result<(), String> {
    let path_str = match name {
        "write_file" | "edit_file" | "write_spreadsheet" | "write_document" => {
            args["path"].as_str().unwrap_or("")
        }
        "process_image" => args["output_path"].as_str().unwrap_or(""),
        _ => "",
    };
    if !path_str.is_empty() {
        let p = std::path::Path::new(path_str);
        let resolved = if p.is_absolute() {
            p.to_path_buf()
        } else {
            working_dir.join(p)
        };
        write_guard.check_write(&resolved)?;
    }
    Ok(())
}

/// Enregistre les fichiers "vus" après un tool read-only réussi.
/// - read_file/read_document/read_spreadsheet/read_image/process_image → le chemin lu.
/// - grep/glob/list_dir → tous les fichiers retournés dans le résultat.
pub fn post_record_read(
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    tr: &ToolResult,
    write_guard: &mut WriteGuard,
) {
    if tr.is_error {
        return;
    }
    match name {
        "read_file" | "read_document" | "read_spreadsheet" | "read_image" => {
            if let Some(path_str) = args["path"].as_str() {
                record_path(write_guard, path_str, working_dir);
            }
        }
        "process_image" => {
            if let Some(path_str) = args["input_path"].as_str() {
                record_path(write_guard, path_str, working_dir);
            }
        }
        "grep" | "glob" | "list_dir" => {
            let seen = super::write_guard_extract::extract_seen_paths(
                name,
                args,
                working_dir,
                &tr.content,
            );
            if !seen.is_empty() {
                write_guard.record_reads(&seen);
            }
        }
        _ => {}
    }
}

/// Enregistre un fichier comme "déjà lu" après qu'il a été écrit/édité.
/// Évite le blocage au tour suivant sur un fichier que l'IA vient de modifier.
pub fn post_record_write(
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    tr: &ToolResult,
    write_guard: &mut WriteGuard,
) {
    if tr.is_error {
        return;
    }
    if name == "bash" && !tr.affected_paths.is_empty() {
        let paths = tr
            .affected_paths
            .iter()
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>();
        write_guard.record_reads(&paths);
        return;
    }
    let path_str = match name {
        "write_file" | "edit_file" | "write_spreadsheet" | "write_document" => {
            args["path"].as_str()
        }
        "process_image" => args["output_path"].as_str(),
        _ => return,
    };
    if let Some(path_str) = path_str {
        record_path(write_guard, path_str, working_dir);
    }
}

fn record_path(write_guard: &mut WriteGuard, path_str: &str, working_dir: &std::path::Path) {
    let p = std::path::Path::new(path_str);
    let resolved = if p.is_absolute() {
        p.to_path_buf()
    } else {
        working_dir.join(p)
    };
    write_guard.record_read(&resolved);
}

/// Résout le chemin d'un outil fichier pour l'affichage frontend.
/// Retourne le chemin absolu résolu (working_dir + chemin relatif) pour les outils
/// qui manipulent un fichier. Retourne None pour les outils sans fichier.
pub fn resolve_tool_path(
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
) -> Option<String> {
    let path_str = match name {
        "read_file" | "write_file" | "edit_file" | "read_spreadsheet" | "read_document"
        | "read_image" | "write_spreadsheet" | "write_document" => args["path"].as_str(),
        "process_image" => args["input_path"].as_str(),
        _ => return None,
    }?;
    let p = std::path::Path::new(path_str);
    let resolved = if p.is_absolute() {
        p.to_path_buf()
    } else {
        working_dir.join(p)
    };
    resolved.to_str().map(|s| s.to_string())
}

pub async fn dispatch_or_interactive(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    if super::tool_catalog::is_optional_tool(name)
        && !super::agent_settings::is_tool_enabled(name).await
    {
        return ToolResult::err("Outil désactivé dans les paramètres.");
    }
    if name == "ask_user_choice" {
        return super::tool_interactive::execute(args, on_event, cancel, Some(session_id)).await;
    }
    if name == "planmode" {
        return super::tool_plan::execute(args, on_event, session_id, cancel).await;
    }
    if name == "exitplanmode" {
        return super::tool_plan::execute_exit(args, on_event, session_id).await;
    }
    super::tool_dispatcher::dispatch(name, args, working_dir, session_id, cancel).await
}
