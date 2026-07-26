use crate::services::agent_local::security;
use crate::services::agent_local::tool_scan_timeout::{run_scan, scan_cancelled};
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

    if let Err(e) = security::validate_read_path(&root, working_dir) {
        return ToolResult::err(e);
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
    if pattern.len() > MAX_PATTERN_LEN {
        return ToolResult::err(format!("Pattern trop long (max {MAX_PATTERN_LEN} chars)"));
    }
    let matcher = match RegexMatcher::new(pattern) {
        Ok(m) => m,
        Err(e) => return ToolResult::err(format!("Pattern regex invalide : {e}")),
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
            return ToolResult::err(format!("Glob invalide : {e}"));
        }
        match ov.build() {
            Ok(overrides) => {
                walk_builder.overrides(overrides);
            }
            Err(e) => return ToolResult::err(format!("Glob invalide : {e}")),
        }
    }

    let mut searcher = Searcher::new();
    let mut results: Vec<String> = Vec::new();
    let mut skipped_errors = 0usize;
    let mut truncated = false;

    for dent in walk_builder.build() {
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
        let mut sink = MatchSink {
            path,
            results: &mut results,
            max: MAX_RESULTS,
        };
        if searcher.search_path(&matcher, path, &mut sink).is_err() {
            skipped_errors = skipped_errors.saturating_add(1);
        }
        if results.len() >= MAX_RESULTS {
            truncated = true;
            break;
        }
    }

    let mut output = results.join("\n");
    if truncated {
        output.push_str(&format!("\n... [tronqué à {MAX_RESULTS} lignes]"));
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
