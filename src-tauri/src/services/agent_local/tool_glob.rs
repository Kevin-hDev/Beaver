use crate::services::agent_local::security;
use crate::services::agent_local::tool_scan_timeout::{run_scan, scan_cancelled};
use crate::services::agent_local::types_tools::ToolResult;
use globset::Glob;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

const MAX_RESULTS: usize = 100;

pub async fn glob_files(pattern: &str, path: Option<&str>, working_dir: &Path) -> ToolResult {
    let pattern = pattern.to_string();
    let root = resolve_root(path, working_dir);

    if let Err(e) = security::validate_read_path(&root, working_dir) {
        return ToolResult::err(e);
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
        Err(e) => return ToolResult::err(format!("Pattern glob invalide : {e}")),
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
    let mut truncated = false;

    for dent in walk {
        if scan_cancelled(cancelled) {
            return ToolResult::err("Timeout après 600s");
        }
        if results.len() >= MAX_RESULTS {
            truncated = true;
            break;
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
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path);
        if matcher.is_match(rel) {
            results.push(path.display().to_string());
        }
    }

    let mut output = results.join("\n");
    if truncated {
        output.push_str(&format!("\n... [tronqué à {MAX_RESULTS} résultats]"));
    }
    if skipped_errors > 0 {
        output.push_str(&format!(
            "\n... [{skipped_errors} erreur(s) de lecture ignorée(s)]"
        ));
    }
    if output.is_empty() {
        output = "(aucun résultat)".into();
    }
    ToolResult::ok(output)
}
