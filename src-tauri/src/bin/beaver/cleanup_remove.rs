use super::{is_safely_inside, Candidate, CandidateKind};
use std::path::{Path, PathBuf};

pub(super) struct RemovalSummary {
    pub removed: usize,
    pub failed: Vec<PathBuf>,
    pub recovered: u64,
}

pub(super) fn remove_candidates(root: &Path, candidates: &[Candidate]) -> RemovalSummary {
    let mut removed = 0_usize;
    let mut failed = Vec::new();
    let mut recovered = 0_u64;
    let mut tool_paths = Vec::new();
    let mut ollama_paths = Vec::new();
    for candidate in candidates {
        if !is_safely_inside(&candidate.family, &candidate.path) {
            failed.push(candidate.path.clone());
            continue;
        }
        match candidate.kind {
            CandidateKind::File => match std::fs::remove_file(&candidate.path) {
                Ok(()) => {
                    removed += 1;
                    recovered = recovered.saturating_add(candidate.bytes);
                }
                Err(_) => failed.push(candidate.path.clone()),
            },
            CandidateKind::ToolResult => tool_paths.push(candidate.path.clone()),
            CandidateKind::OllamaStaging => ollama_paths.push(candidate.path.clone()),
        }
    }
    for (paths, outcome) in [
        (
            &tool_paths,
            cl_go_dash_lib::cli_support::remove_tool_results(&tool_paths),
        ),
        (
            &ollama_paths,
            cl_go_dash_lib::cli_support::remove_abandoned_ollama_staging(root, &ollama_paths),
        ),
    ] {
        removed += outcome.removed;
        recovered = recovered.saturating_add(
            candidates
                .iter()
                .filter(|candidate| {
                    paths.contains(&candidate.path) && !outcome.failed.contains(&candidate.path)
                })
                .map(|candidate| candidate.bytes)
                .fold(0_u64, u64::saturating_add),
        );
        failed.extend(outcome.failed);
    }
    RemovalSummary {
        removed,
        failed,
        recovered,
    }
}
