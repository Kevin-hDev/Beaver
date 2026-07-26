use git2::Repository;
use std::path::{Path, PathBuf};

pub fn open(repo_path: &Path) -> Result<Repository, String> {
    Repository::discover(repo_path).map_err(|_| "Dépôt git introuvable".to_string())
}

pub fn workdir(repo: &Repository) -> Result<PathBuf, String> {
    repo.workdir()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Dépôt git invalide".to_string())
}

pub fn has_github_remote(repo: &Repository) -> bool {
    let Ok(remotes) = repo.remotes() else {
        return false;
    };
    for name in remotes
        .iter()
        .filter_map(|name| name.ok().flatten())
        .take(32)
    {
        let Ok(remote) = repo.find_remote(name) else {
            continue;
        };
        if remote.url().ok().is_some_and(is_github_url)
            || remote.pushurl().ok().flatten().is_some_and(is_github_url)
        {
            return true;
        }
    }
    false
}

pub(super) fn is_github_url(url: &str) -> bool {
    url.starts_with("https://github.com/")
        || url.starts_with("ssh://git@github.com/")
        || url.starts_with("git@github.com:")
}

#[cfg(test)]
mod tests {
    use super::open;
    use git2::{Repository, Signature};
    use std::path::Path;

    #[test]
    fn opens_repo_from_nested_directory() {
        let tmp = tempfile::tempdir().expect("temp repo");
        let repo = Repository::init(tmp.path()).expect("init");
        std::fs::create_dir_all(tmp.path().join("nested")).expect("nested");
        std::fs::write(tmp.path().join("file.txt"), "data").expect("file");
        commit_file(&repo);

        let found = open(&tmp.path().join("nested")).expect("discover repo");
        let found_root = std::fs::canonicalize(found.workdir().unwrap()).expect("found root");
        let expected_root = std::fs::canonicalize(tmp.path()).expect("expected root");
        assert_eq!(found_root, expected_root);
    }

    fn commit_file(repo: &Repository) {
        let mut index = repo.index().expect("index");
        index.add_path(Path::new("file.txt")).expect("add");
        index.write().expect("write");
        let tree_oid = index.write_tree().expect("tree");
        let tree = repo.find_tree(tree_oid).expect("tree");
        let sig = Signature::now("CL-GO Test", "test@example.com").expect("sig");
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .expect("commit");
    }
}
