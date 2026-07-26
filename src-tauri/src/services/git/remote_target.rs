use git2::{Branch, Repository};

const MAX_REMOTES: usize = 32;

pub(super) fn current_branch(repository: &Repository) -> Result<Branch<'_>, String> {
    let head = repository
        .head()
        .map_err(|_| "Branche introuvable".to_string())?;
    if !head.is_branch() {
        return Err("Branche introuvable".to_string());
    }
    Ok(Branch::wrap(head))
}

pub(super) fn branch_name(branch: &Branch<'_>) -> Result<String, String> {
    branch
        .name()
        .ok()
        .flatten()
        .map(str::to_string)
        .ok_or_else(|| "Branche introuvable".to_string())
}

pub(super) fn choose_remote(repository: &Repository, branch: &str) -> Option<String> {
    if let Ok(name) = repository
        .config()
        .and_then(|config| config.get_string(&format!("branch.{branch}.remote")))
    {
        if name != "." && repository.find_remote(&name).is_ok() {
            return Some(name);
        }
    }
    if repository.find_remote("origin").is_ok() {
        return Some("origin".to_string());
    }
    repository.remotes().ok().and_then(|names| {
        names
            .iter()
            .filter_map(|name| name.ok().flatten())
            .take(MAX_REMOTES)
            .next()
            .map(str::to_string)
    })
}

pub(super) fn selected_remote_url(repository: &Repository, branch: &str) -> Option<String> {
    let remote_name = choose_remote(repository, branch)?;
    let remote = repository.find_remote(&remote_name).ok()?;
    remote
        .pushurl()
        .ok()
        .flatten()
        .or_else(|| remote.url().ok())
        .map(str::to_string)
}
