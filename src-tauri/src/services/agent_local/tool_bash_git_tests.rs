use super::GitBaseline;

#[test]
fn git_status_ignores_generated_dependency_directories() {
    let directory = tempfile::tempdir().expect("tempdir");
    let repository = git2::Repository::init(directory.path()).expect("repo");
    let source = directory.path().join("src/main.rs");
    let dependency = directory.path().join("node_modules/pkg/index.js");
    std::fs::create_dir_all(source.parent().expect("source parent")).expect("source directory");
    std::fs::create_dir_all(dependency.parent().expect("dependency parent"))
        .expect("dependency directory");
    std::fs::write(&source, "fn main() {}\n").expect("source file");
    std::fs::write(&dependency, "module.exports = true;\n").expect("dependency file");

    let root = dunce::canonicalize(directory.path()).expect("canonical root");
    let (paths, incomplete) = super::status_paths(&repository, &root);
    let source = dunce::canonicalize(source).expect("canonical source");
    let dependency_root = dunce::canonicalize(directory.path().join("node_modules"))
        .expect("canonical dependency root");

    assert!(!incomplete);
    assert!(
        paths.iter().any(|(path, _)| path == &source),
        "tracked paths: {paths:?}"
    );
    assert!(paths
        .iter()
        .all(|(path, _)| !path.starts_with(&dependency_root)));
}

#[test]
fn baseline_uses_the_exact_dirty_worktree_content() {
    let directory = tempfile::tempdir().expect("tempdir");
    let repository = git2::Repository::init(directory.path()).expect("repo");
    let file = directory.path().join("file.txt");
    std::fs::write(&file, "before").expect("write before");
    commit_all(&repository, "initial");
    std::fs::write(&file, "dirty before command").expect("dirty");

    let (baseline, incomplete) = GitBaseline::capture(directory.path());
    assert!(!incomplete);
    let baseline = baseline.expect("baseline");
    let repository = baseline.open_repository().expect("open repository");
    std::fs::write(&file, "after command").expect("after");
    let before = baseline
        .before_state(Some(&repository), &file)
        .expect("known path")
        .expect("existing file");
    let mut remaining = super::MAX_FILE_CHANGE_DIFF_BYTES;
    let after = super::capture(&file, &mut remaining).expect("after state");
    let change = super::super::tool_file_changes::build_change(
        &file,
        Some(&before),
        Some(&after),
    )
    .expect("change");

    let diff = change.diff.expect("diff");
    let lines = diff
        .hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .map(|line| line.content.as_str())
        .collect::<Vec<_>>();
    assert!(lines.iter().any(|line| line.contains("dirty before command")));
    assert!(lines.iter().any(|line| line.contains("after command")));
}

fn commit_all(repository: &git2::Repository, message: &str) {
    let mut index = repository.index().expect("index");
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .expect("add");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("tree id");
    let tree = repository.find_tree(tree_id).expect("tree");
    let signature = git2::Signature::now("Beaver", "beaver@example.test").expect("signature");
    repository
        .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
        .expect("commit");
}
