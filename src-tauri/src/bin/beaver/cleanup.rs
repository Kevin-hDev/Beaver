use crate::output::{confirmation, format_size, Out};
use std::path::PathBuf;
use std::time::SystemTime;

#[path = "cleanup_inventory.rs"]
mod cleanup_inventory;
#[path = "cleanup_remove.rs"]
mod cleanup_remove;

pub use cleanup_inventory::{inventory, is_safely_inside};

#[derive(Clone, Copy)]
pub(super) enum CandidateKind {
    File,
    ToolResult,
    OllamaStaging,
}

pub struct Candidate {
    pub path: PathBuf,
    pub bytes: u64,
    pub reason_fr: &'static str,
    pub reason_en: &'static str,
    pub(super) family: PathBuf,
    pub(super) kind: CandidateKind,
}

#[derive(Debug)]
pub struct InventoryError {
    pub(super) family: &'static str,
}

fn display_inventory(out: &Out, candidates: &[Candidate]) {
    for candidate in candidates {
        out.line(&format!(
            "  {} — {}",
            out.t(candidate.reason_fr, candidate.reason_en),
            format_size(candidate.bytes)
        ));
    }
    let total = candidates
        .iter()
        .map(|candidate| candidate.bytes)
        .fold(0_u64, u64::saturating_add);
    out.line(&format!(
        "{} {}",
        out.t("Total :", "Total:"),
        format_size(total)
    ));
}

pub fn run(out: &Out, args: &[String]) -> i32 {
    if crate::app_detect::app_is_running() {
        out.line(out.t("Fermez Beaver d’abord.", "Close Beaver first."));
        return 1;
    }
    let dry_run = match args {
        [] => false,
        [flag] if flag == "--dry-run" => true,
        _ => {
            out.line(out.t(
                "Usage : beaver cleanup [--dry-run]",
                "Usage: beaver cleanup [--dry-run]",
            ));
            return 2;
        }
    };
    let root = cl_go_dash_lib::cli_support::data_dir();
    let candidates = match inventory(&root, SystemTime::now()) {
        Ok(candidates) => candidates,
        Err(error) => {
            out.line(&format!(
                "{} {}.",
                out.t("Inventaire impossible pour", "Inventory unavailable for"),
                error.family
            ));
            return 1;
        }
    };
    if candidates.is_empty() {
        out.line(out.t("Rien à nettoyer.", "Nothing to clean."));
        return 0;
    }
    display_inventory(out, &candidates);
    if dry_run {
        return 0;
    }
    if !confirmation(
        out,
        "Supprimer ces éléments ? [o/N]",
        "Delete these items? [y/N]",
    ) {
        out.line(out.t("Rien n’a été supprimé.", "Nothing was deleted."));
        return 0;
    }
    let summary = cleanup_remove::remove_candidates(&root, &candidates);
    for path in &summary.failed {
        if let Some(candidate) = candidates.iter().find(|candidate| candidate.path == *path) {
            out.line(&format!(
                "  ✗ {}",
                out.t(candidate.reason_fr, candidate.reason_en)
            ));
        }
    }
    out.line(&format!(
        "{} {}, {} {}, {} {}",
        out.t("Supprimés :", "Removed:"),
        summary.removed,
        out.t("échoués :", "failed:"),
        summary.failed.len(),
        out.t("récupérés :", "recovered:"),
        format_size(summary.recovered)
    ));
    i32::from(!summary.failed.is_empty())
}

#[cfg(test)]
#[path = "cleanup_tests.rs"]
mod tests;
