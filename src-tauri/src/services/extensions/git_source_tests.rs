use super::npm_runner::NpmRunner;
use super::source_validation::GitSource;
use git2::{ObjectType, Repository, Signature};
use std::path::{Path, PathBuf};

fn create_repository(path: &Path) -> String {
    let repository = Repository::init(path).unwrap();
    std::fs::write(path.join("index.ts"), "export default () => {}").unwrap();
    std::fs::write(
        path.join("beaver-extension.json"),
        serde_json::json!({
            "id": "test.git.install",
            "name": "Git test",
            "version": "1.0.0",
            "beaverApi": "1",
            "runtime": "node",
            "main": "index.ts",
            "access": "full"
        })
        .to_string(),
    )
    .unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("index.ts")).unwrap();
    index.add_path(Path::new("beaver-extension.json")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repository.find_tree(tree_id).unwrap();
    let signature = Signature::now("Beaver test", "test@beaver.local").unwrap();
    repository
        .commit(Some("HEAD"), &signature, &signature, "fixture", &tree, &[])
        .unwrap()
        .to_string()
}

#[test]
fn git_materialization_is_clean_and_reports_the_revision() {
    let temporary = tempfile::tempdir().unwrap();
    let repository_path = temporary.path().join("repository");
    let destination = temporary.path().join("destination");
    std::fs::create_dir(&repository_path).unwrap();
    std::fs::create_dir(&destination).unwrap();
    let expected_revision = create_repository(&repository_path);
    let npm = NpmRunner::for_test(PathBuf::from("/node"), PathBuf::from("/npm-cli.js"));
    let source = GitSource {
        locator: "https://example.invalid/extension.git".to_string(),
        clone_url: url::Url::from_file_path(&repository_path)
            .unwrap()
            .to_string(),
        reference: None,
    };
    let probe = temporary.path().join("probe");
    super::git_source::clone_repository(&source, &probe).unwrap();
    std::fs::remove_dir_all(probe).unwrap();

    let materialized = super::git_source::materialize(&source, &destination, &npm).unwrap();

    assert_eq!(materialized.revision, expected_revision);
    assert!(!materialized.root.join(".git").exists());
    assert!(materialized.root.join("beaver-extension.json").is_file());
}

#[test]
fn git_materialization_accepts_a_declared_tag() {
    let temporary = tempfile::tempdir().unwrap();
    let repository_path = temporary.path().join("repository");
    std::fs::create_dir(&repository_path).unwrap();
    let expected_revision = create_repository(&repository_path);
    let repository = Repository::open(&repository_path).unwrap();
    let commit = repository
        .revparse_single("HEAD")
        .unwrap()
        .peel(ObjectType::Commit)
        .unwrap();
    repository
        .tag_lightweight("v1.0.0", &commit, false)
        .unwrap();
    drop(commit);
    drop(repository);
    let source = GitSource {
        locator: "https://example.invalid/extension.git#v1.0.0".to_string(),
        clone_url: url::Url::from_file_path(&repository_path)
            .unwrap()
            .to_string(),
        reference: Some("v1.0.0".to_string()),
    };

    let cloned =
        super::git_source::clone_repository(&source, &temporary.path().join("tag-checkout"))
            .unwrap();

    assert_eq!(
        cloned
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string(),
        expected_revision
    );
}

#[test]
fn git_materialization_accepts_a_declared_branch() {
    let temporary = tempfile::tempdir().unwrap();
    let repository_path = temporary.path().join("repository");
    std::fs::create_dir(&repository_path).unwrap();
    let expected_revision = create_repository(&repository_path);
    let repository = Repository::open(&repository_path).unwrap();
    let commit = repository.head().unwrap().peel_to_commit().unwrap();
    repository.branch("feature", &commit, false).unwrap();
    drop(commit);
    drop(repository);
    let source = GitSource {
        locator: "https://example.invalid/extension.git#feature".to_string(),
        clone_url: url::Url::from_file_path(&repository_path)
            .unwrap()
            .to_string(),
        reference: Some("feature".to_string()),
    };

    let cloned =
        super::git_source::clone_repository(&source, &temporary.path().join("branch-checkout"))
            .unwrap();

    assert_eq!(
        cloned
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string(),
        expected_revision
    );
}

#[test]
fn git_materialization_accepts_a_pinned_commit() {
    let temporary = tempfile::tempdir().unwrap();
    let repository_path = temporary.path().join("repository");
    std::fs::create_dir(&repository_path).unwrap();
    let expected_revision = create_repository(&repository_path);
    let repository = Repository::open(&repository_path).unwrap();
    std::fs::write(repository_path.join("index.ts"), "export default () => 2").unwrap();
    let mut index = repository.index().unwrap();
    index.add_path(Path::new("index.ts")).unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repository.find_tree(tree_id).unwrap();
    let parent = repository.head().unwrap().peel_to_commit().unwrap();
    let signature = Signature::now("Beaver test", "test@beaver.local").unwrap();
    repository
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            "newer fixture",
            &tree,
            &[&parent],
        )
        .unwrap();
    drop(parent);
    drop(tree);
    drop(repository);
    let source = GitSource {
        locator: format!("https://example.invalid/extension.git#{expected_revision}"),
        clone_url: url::Url::from_file_path(&repository_path)
            .unwrap()
            .to_string(),
        reference: Some(expected_revision.clone()),
    };

    let cloned =
        super::git_source::clone_repository(&source, &temporary.path().join("commit-checkout"))
            .unwrap();

    assert_eq!(
        cloned
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string(),
        expected_revision
    );
}
