use super::limits::{MAX_INSTRUCTION_BYTES, MAX_PATH_BYTES};
use std::path::{Path, PathBuf};

const MAX_OPENCLAW_WORKSPACES: usize = 3;

pub fn env_root(name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os(name)?);
    valid_root(&path).then_some(path)
}

pub fn openclaw_workspaces(root: &Path, home: &Path) -> Vec<PathBuf> {
    let mut workspaces = vec![root.join("workspace")];
    add_workspace_variants(root, &mut workspaces);
    add_configured_workspaces(root, home, &mut workspaces);
    workspaces.dedup();
    workspaces.truncate(MAX_OPENCLAW_WORKSPACES);
    workspaces
}

fn add_workspace_variants(root: &Path, workspaces: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let limit = MAX_OPENCLAW_WORKSPACES.saturating_sub(1);
    let mut variants = Vec::with_capacity(limit);
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        let matches = path.is_dir()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("workspace-"));
        if matches {
            variants.push(path);
            variants.sort();
            variants.truncate(limit);
        }
    }
    workspaces.extend(variants);
}

fn add_configured_workspaces(root: &Path, home: &Path, workspaces: &mut Vec<PathBuf>) {
    let Ok(bytes) = std::fs::read(root.join("openclaw.json")) else {
        return;
    };
    if bytes.len() > MAX_INSTRUCTION_BYTES as usize {
        return;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return;
    };
    add_configured_workspace(
        workspaces,
        value.pointer("/agents/defaults/workspace"),
        home,
    );
    if let Some(agents) = value
        .pointer("/agents/list")
        .and_then(|entry| entry.as_array())
    {
        for agent in agents {
            if workspaces.len() >= MAX_OPENCLAW_WORKSPACES {
                break;
            }
            add_configured_workspace(workspaces, agent.get("workspace"), home);
        }
    }
}

fn add_configured_workspace(
    workspaces: &mut Vec<PathBuf>,
    value: Option<&serde_json::Value>,
    home: &Path,
) {
    let Some(value) = value.and_then(|entry| entry.as_str()) else {
        return;
    };
    let path = expand_home(value, home);
    if valid_root(&path) && !workspaces.contains(&path) {
        workspaces.push(path);
    }
}

fn valid_root(path: &Path) -> bool {
    path.is_absolute()
        && path.as_os_str().len() <= MAX_PATH_BYTES
        && !path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
}

fn expand_home(value: &str, home: &Path) -> PathBuf {
    value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
        .map(|rest| home.join(rest))
        .unwrap_or_else(|| PathBuf::from(value))
}
