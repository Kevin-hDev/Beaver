use git2::{Repository, StatusOptions, StatusShow};
use std::path::{Path, PathBuf};

const MAX_STATUS_CHARS: usize = 1500;
const MAX_SECTION_CHARS: usize = 2000;
const MAX_COMMITS: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct GitSnapshot {
    pub is_git: bool,
    pub current_branch: Option<String>,
    pub default_branch: Option<String>,
    pub status_short: Option<String>,
    pub recent_commits: Option<String>,
    pub git_root: Option<PathBuf>,
}

pub fn detect_git(path: &Path) -> GitSnapshot {
    let repo = match Repository::discover(path) {
        Ok(r) => r,
        Err(_) => return GitSnapshot::default(),
    };

    GitSnapshot {
        is_git: true,
        current_branch: detect_current_branch(&repo),
        default_branch: detect_default_branch(&repo),
        status_short: build_status_short(&repo),
        recent_commits: build_recent_commits(&repo),
        git_root: repo.workdir().map(Path::to_path_buf),
    }
}

pub fn format_git_section(snap: &GitSnapshot) -> Option<String> {
    if !snap.is_git {
        return None;
    }

    let mut s = String::from(
        "gitStatus: This is the git status at the start of the conversation.\n\
         Note that this status is a snapshot in time, and will not update \
         during the conversation.",
    );

    if let Some(b) = &snap.current_branch {
        s.push_str(&format!("\n\nCurrent branch: {b}"));
    }
    if let Some(d) = &snap.default_branch {
        s.push_str(&format!(
            "\n\nMain branch (you will usually use this for PRs): {d}"
        ));
    }
    if let Some(st) = &snap.status_short {
        if !st.is_empty() {
            s.push_str(&format!("\n\nStatus:\n{st}"));
        }
    }
    if let Some(c) = &snap.recent_commits {
        if !c.is_empty() {
            s.push_str(&format!("\n\nRecent commits:\n{c}"));
        }
    }

    if s.len() > MAX_SECTION_CHARS {
        s.truncate(MAX_SECTION_CHARS - 4);
        s.push_str("\n...");
    }
    Some(s)
}

fn detect_current_branch(repo: &Repository) -> Option<String> {
    if repo.head_detached().unwrap_or(false) {
        let id = repo.head().ok()?.peel_to_commit().ok()?.id().to_string();
        return Some(format!("HEAD detached at {}", &id[..7.min(id.len())]));
    }
    repo.head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(String::from))
}

fn detect_default_branch(repo: &Repository) -> Option<String> {
    if let Ok(r) = repo.find_reference("refs/remotes/origin/HEAD") {
        if let Ok(Some(target)) = r.symbolic_target() {
            let short = target
                .strip_prefix("refs/remotes/origin/")
                .unwrap_or(target);
            return Some(short.to_string());
        }
    }
    if let Ok(cfg) = repo.config() {
        if let Ok(name) = cfg.get_string("init.defaultBranch") {
            return Some(name);
        }
    }
    Some("main".to_string())
}

fn build_status_short(repo: &Repository) -> Option<String> {
    let mut opts = StatusOptions::new();
    opts.show(StatusShow::IndexAndWorkdir)
        .include_untracked(true)
        .recurse_untracked_dirs(false);

    let statuses = repo.statuses(Some(&mut opts)).ok()?;
    if statuses.is_empty() {
        return None;
    }

    let mut result = String::new();
    for entry in statuses.iter() {
        if result.len() >= MAX_STATUS_CHARS {
            result.push_str("...\n");
            break;
        }
        let path = entry.path().unwrap_or("?");
        let prefix = status_prefix(entry.status());
        result.push_str(&format!("{prefix} {path}\n"));
    }
    Some(result.trim_end().to_string())
}

fn status_prefix(flags: git2::Status) -> &'static str {
    if flags.is_index_new() {
        "A "
    } else if flags.is_index_modified() || flags.is_wt_modified() {
        "M "
    } else if flags.is_index_deleted() || flags.is_wt_deleted() {
        "D "
    } else if flags.is_index_renamed() || flags.is_wt_renamed() {
        "R "
    } else if flags.is_wt_new() {
        "??"
    } else {
        "  "
    }
}

fn build_recent_commits(repo: &Repository) -> Option<String> {
    let mut revwalk = repo.revwalk().ok()?;
    revwalk.push_head().ok()?;

    let mut lines = Vec::with_capacity(MAX_COMMITS);
    for oid_result in revwalk.take(MAX_COMMITS) {
        let oid = oid_result.ok()?;
        let commit = repo.find_commit(oid).ok()?;
        let short = &oid.to_string()[..7];
        let summary = commit.summary().ok().flatten().unwrap_or("(no message)");
        lines.push(format!("{short} {summary}"));
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
#[path = "git_context_tests.rs"]
mod tests;
