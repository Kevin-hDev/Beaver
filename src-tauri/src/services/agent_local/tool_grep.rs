use crate::services::agent_local::security;
use crate::services::agent_local::tool_scan_timeout::{run_scan, scan_cancelled};
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;
use grep_regex::RegexMatcher;
use grep_searcher::{Searcher, Sink, SinkMatch};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

const MAX_RESULTS: usize = 250;

pub async fn grep(
    pattern: &str,
    path: Option<&str>,
    glob_filter: Option<&str>,
    working_dir: &Path,
) -> ToolResult {
    let pattern = pattern.to_string();
    let glob_filter = glob_filter.map(|s| s.to_string());
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
        return super::tool_file_error::path_failure(
            security::sanitize_error(error),
            "search_root_not_found",
            "search_path_denied",
            "invalid_search_path",
        );
    }

    run_scan(move |cancelled| grep_blocking(&pattern, &root, glob_filter.as_deref(), &cancelled))
        .await
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

const MAX_PATTERN_LEN: usize = 500;

fn grep_blocking(
    pattern: &str,
    root: &Path,
    glob_filter: Option<&str>,
    cancelled: &AtomicBool,
) -> ToolResult {
    if pattern.chars().count() > MAX_PATTERN_LEN {
        return ToolResult::error(
            format!("Pattern trop long (max {MAX_PATTERN_LEN} chars)"),
            "grep_pattern_too_long",
            ToolErrorCategory::Validation,
            false,
        );
    }
    let matcher = match RegexMatcher::new(pattern) {
        Ok(m) => m,
        Err(e) => {
            return ToolResult::error(
                format!("Pattern regex invalide : {e}"),
                "invalid_grep_pattern",
                ToolErrorCategory::Validation,
                false,
            )
        }
    };

    let mut walk_builder = WalkBuilder::new(root);
    walk_builder
        .hidden(false)
        .parents(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false);

    if let Some(g) = glob_filter {
        let mut ov = OverrideBuilder::new(root);
        if let Err(e) = ov.add(g) {
            return ToolResult::error(
                format!("Glob invalide : {e}"),
                "invalid_grep_glob",
                ToolErrorCategory::Validation,
                false,
            );
        }
        match ov.build() {
            Ok(overrides) => {
                walk_builder.overrides(overrides);
            }
            Err(e) => {
                return ToolResult::error(
                    format!("Glob invalide : {e}"),
                    "invalid_grep_glob",
                    ToolErrorCategory::Validation,
                    false,
                )
            }
        }
    }

    let mut searcher = Searcher::new();
    let mut results: Vec<String> = Vec::new();
    let mut skipped_errors = 0usize;
    let mut scanned_files = 0usize;
    let mut truncated = false;

    for dent in walk_builder.build() {
        if scan_cancelled(cancelled) {
            return ToolResult::error(
                "Timeout après 600s",
                "grep_timeout",
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
        let path = entry.path();
        let mut sink = MatchSink {
            path,
            results: &mut results,
            max: MAX_RESULTS + 1,
        };
        match searcher.search_path(&matcher, path, &mut sink) {
            Ok(()) => scanned_files = scanned_files.saturating_add(1),
            Err(_) => skipped_errors = skipped_errors.saturating_add(1),
        }
        if results.len() > MAX_RESULTS {
            results.truncate(MAX_RESULTS);
            truncated = true;
            break;
        }
    }

    let mut output = results.join("\n");
    if truncated {
        output.push_str(&format!("\n... [tronqué à {MAX_RESULTS} lignes]"));
    }
    if output.is_empty() {
        output = "(aucun résultat)".into();
    }
    if skipped_errors > 0 && scanned_files == 0 {
        return ToolResult::error(
            format!("Aucun fichier lisible; {skipped_errors} erreur(s) de lecture."),
            "grep_scan_failed",
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

struct MatchSink<'a> {
    path: &'a Path,
    results: &'a mut Vec<String>,
    max: usize,
}

impl<'a> Sink for MatchSink<'a> {
    type Error = std::io::Error;

    fn matched(
        &mut self,
        _searcher: &Searcher,
        mat: &SinkMatch<'_>,
    ) -> Result<bool, std::io::Error> {
        if self.results.len() >= self.max {
            return Ok(false);
        }
        let line = mat.line_number().unwrap_or(0);
        let text = String::from_utf8_lossy(mat.bytes());
        let trimmed = text.trim_end_matches(['\n', '\r']);
        self.results
            .push(format!("{}:{}:{}", self.path.display(), line, trimmed));
        Ok(self.results.len() < self.max)
    }
}
