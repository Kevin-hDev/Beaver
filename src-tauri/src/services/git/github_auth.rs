use git2::Repository;

use super::repo;

pub const GITHUB_AUTH_REQUIRED: &str = "GITHUB_AUTH_REQUIRED";

pub fn ensure_branch_creation_allowed(repo: &Repository) -> Result<(), String> {
    if branch_creation_requires_auth(repo::has_github_remote(repo), github_connected()) {
        return Err(GITHUB_AUTH_REQUIRED.to_string());
    }
    Ok(())
}

fn github_connected() -> bool {
    let configured = crate::services::mcp_bridge::config::find("github")
        .ok()
        .flatten()
        .map(|c| c.status == "connected")
        .unwrap_or(false);
    configured && crate::services::mcp_oauth::storage::has_tokens("github")
}

pub(super) fn branch_creation_requires_auth(has_github_remote: bool, connected: bool) -> bool {
    has_github_remote && !connected
}

#[cfg(test)]
mod tests {
    use super::branch_creation_requires_auth;
    use crate::services::git::repo::is_github_url;

    #[test]
    fn detects_github_remotes() {
        assert!(is_github_url("https://github.com/Kevin-hDev/TEST.git"));
        assert!(is_github_url("git@github.com:Kevin-hDev/TEST.git"));
        assert!(is_github_url("ssh://git@github.com/Kevin-hDev/TEST.git"));
        assert!(!is_github_url("https://gitlab.com/Kevin-hDev/TEST.git"));
    }

    #[test]
    fn requires_auth_only_for_unconnected_github_repo() {
        assert!(branch_creation_requires_auth(true, false));
        assert!(!branch_creation_requires_auth(true, true));
        assert!(!branch_creation_requires_auth(false, false));
    }
}
