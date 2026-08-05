use crate::services::agent_local::security;
use crate::services::agent_local::tool_scan_timeout::{run_scan, scan_cancelled};
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;
use globset::Glob;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

const MAX_RESULTS: usize = 100;

pub async fn glob_files(pattern: &str, path: Option<&str>, working_dir: &Path) -> ToolResult {
    let pattern = pattern.to_string();
    let root = resolve_root(path, working_dir);

    if let Err(error) = security::validate_read_path(&root, working_dir) {
        return super::tool_file_error::path_failure(
            error,
            "search_root_not_found",
            "search_path_denied",
            "invalid_search_path",
        );
    }
    if let Err(error) = tokio::fs::metadata(&root).await {
        return super::tool_file_error::search_root_failure(error);
    }

    run_scan(move |cancelled| glob_blocking(&pattern, &root, &cancelled)).await
}

fn resolve_root(path: Option<&str>, working_dir: &Path) -> PathBuf {
    match path {
        Some(p) => {
            let pb = Path::new(p);
            if pb.is_absolute() {
                pb.to_path_buf()
            } else {
                working_dir.join(pb)
            }
        }
        None => working_dir.to_path_buf(),
    }
}

fn glob_blocking(pattern: &str, root: &Path, cancelled: &AtomicBool) -> ToolResult {
    let matcher = match Glob::new(pattern) {
        Ok(g) => g.compile_matcher(),
        Err(e) => {
            return ToolResult::error(
                format!("Pattern glob invalide : {e}"),
                "invalid_glob_pattern",
                ToolErrorCategory::Validation,
                false,
            )
        }
    };

    let walk = WalkBuilder::new(root)
        .hidden(false)
        .parents(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .build();

    let mut results: Vec<String> = Vec::new();
    let mut skipped_errors = 0usize;
    let mut scanned_files = 0usize;
    let mut truncated = false;

    for dent in walk {
        if scan_cancelled(cancelled) {
            return ToolResult::error(
                "Timeout après 600s",
                "glob_timeout",
                ToolErrorCategory::Timeout,
                true,
            );
        }
        let entry = match dent {
            Ok(e) => e,
            Err(_) => {
                skipped_errors = skipped_errors.saturating_add(1);
                continue;
            }
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        scanned_files = scanned_files.saturating_add(1);
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        if matcher.is_match(rel) {
            results.push(path.display().to_string());
            if results.len() > MAX_RESULTS {
                results.truncate(MAX_RESULTS);
                truncated = true;
                break;
            }
        }
    }

    let mut output = results.join("\n");
    if truncated {
        output.push_str(&format!("\n... [tronqué à {MAX_RESULTS} résultats]"));
    }
    if output.is_empty() {
        output = "(aucun résultat)".into();
    }
    if skipped_errors > 0 && scanned_files == 0 {
        return ToolResult::error(
            format!("Aucun fichier lisible; {skipped_errors} erreur(s) de lecture."),
            "glob_scan_failed",
            ToolErrorCategory::Execution,
            true,
        );
    }
    let mut result = if skipped_errors > 0 {
        ToolResult::partial(
            output,
            [format!("{skipped_errors} fichier(s) ou dossier(s) n'ont pas pu être lus.")],
        )
    } else {
        ToolResult::ok(output)
    };
    result.mark_truncated(truncated);
    result
}
