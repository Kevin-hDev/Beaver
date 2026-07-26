use super::branch;

fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = git2::Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("file.txt"), "initial").expect("write file");
    let mut index = repo.index().expect("index");
    index
        .add_path(std::path::Path::new("file.txt"))
        .expect("add");
    index.write().expect("write index");
    let tree_oid = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_oid).expect("find tree");
    let sig = git2::Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    drop(tree);
    drop(repo);
    tmp
}

#[test]
fn test_create_branch_rejects_dotdot() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "../../evil");
    assert!(result.is_err(), "branch name with '..' must be rejected");
}

#[test]
fn test_create_branch_rejects_null_byte() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "branch\0evil");
    assert!(
        result.is_err(),
        "branch name with null byte must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_leading_dash() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "-evil");
    assert!(
        result.is_err(),
        "branch name starting with '-' must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_spaces() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "my branch");
    assert!(result.is_err(), "branch name with spaces must be rejected");
}

#[test]
fn test_create_branch_rejects_too_long() {
    let tmp = std::env::temp_dir();
    let name = "a".repeat(101);
    let result = branch::create_branch(&tmp, &name);
    assert!(
        result.is_err(),
        "branch name over 100 chars must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_backslash() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "foo\\bar");
    assert!(
        result.is_err(),
        "branch name with backslash must be rejected"
    );
}

#[test]
fn test_checkout_rejects_dotdot() {
    let tmp = std::env::temp_dir();
    let result = branch::checkout_branch(&tmp, "../../evil");
    assert!(result.is_err(), "branch name with '..' must be rejected");
}

#[test]
fn test_checkout_rejects_null_byte() {
    let tmp = std::env::temp_dir();
    let result = branch::checkout_branch(&tmp, "branch\0evil");
    assert!(
        result.is_err(),
        "branch name with null byte must be rejected"
    );
}

#[test]
fn test_checkout_rejects_backslash() {
    let tmp = std::env::temp_dir();
    let result = branch::checkout_branch(&tmp, "foo\\bar");
    assert!(
        result.is_err(),
        "branch name with backslash must be rejected"
    );
}

#[test]
fn test_get_context_non_git_dir() {
    let tmp = std::env::temp_dir();
    let ctx = branch::get_context(&tmp);
    assert!(!ctx.is_git_repo, "temp dir should not be a git repo");
}

#[test]
fn test_list_branches_non_git_dir() {
    let tmp = std::env::temp_dir();
    let result = branch::list_branches(&tmp);
    assert!(
        result.is_err(),
        "listing branches on non-git dir should fail"
    );
}

#[test]
fn test_create_branch_with_dirty_worktree_preserves_changes() {
    let tmp = init_repo_with_commit();
    std::fs::write(tmp.path().join("file.txt"), "dirty").expect("dirty file");

    let result = branch::create_branch(tmp.path(), "Agentic");

    assert!(
        result.is_ok(),
        "dirty worktree should not block a new branch"
    );
    let ctx = branch::get_context(tmp.path());
    assert_eq!(ctx.branch, "Agentic");
    let content = std::fs::read_to_string(tmp.path().join("file.txt")).expect("read file");
    assert_eq!(content, "dirty");
}

#[test]
fn test_dirty_count_treats_untracked_directory_as_one_entry() {
    let tmp = init_repo_with_commit();
    let generated = tmp.path().join("generated");
    std::fs::create_dir_all(&generated).expect("create generated dir");
    std::fs::write(generated.join("one.txt"), "one").expect("write one");
    std::fs::write(generated.join("two.txt"), "two").expect("write two");

    let branches = branch::list_branches(tmp.path()).expect("list branches");
    let main = branches
        .iter()
        .find(|b| b.name == "master" || b.name == "main")
        .expect("current branch");

    assert_eq!(
        main.dirty_count, 1,
        "un dossier non suivi doit compter comme une seule entrée"
    );
}

#[test]
fn test_create_branch_rejects_colon() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat:bad");
    assert!(result.is_err(), "branch name with ':' must be rejected");
}

#[test]
fn test_create_branch_rejects_tilde() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat~1");
    assert!(result.is_err(), "branch name with '~' must be rejected");
}

#[test]
fn test_create_branch_rejects_caret() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat^bad");
    assert!(result.is_err(), "branch name with '^' must be rejected");
}

#[test]
fn test_create_branch_rejects_question_mark() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat?bad");
    assert!(result.is_err(), "branch name with '?' must be rejected");
}

#[test]
fn test_create_branch_rejects_asterisk() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat*bad");
    assert!(result.is_err(), "branch name with '*' must be rejected");
}

#[test]
fn test_create_branch_rejects_bracket() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat[bad");
    assert!(result.is_err(), "branch name with '[' must be rejected");
}

#[test]
fn test_create_branch_rejects_at_brace() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat@{bad");
    assert!(result.is_err(), "branch name with '@{{' must be rejected");
}

#[test]
fn test_create_branch_rejects_dot_lock_suffix() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat.lock");
    assert!(
        result.is_err(),
        "branch name ending with '.lock' must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_leading_dot() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, ".hidden");
    assert!(
        result.is_err(),
        "branch name starting with '.' must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_trailing_slash() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat/");
    assert!(
        result.is_err(),
        "branch name ending with '/' must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_double_slash() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat//bad");
    assert!(result.is_err(), "branch name with '//' must be rejected");
}

#[test]
fn test_create_branch_rejects_control_chars() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat\x01bad");
    assert!(
        result.is_err(),
        "branch name with control chars must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_del_char() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat\x7fbad");
    assert!(
        result.is_err(),
        "branch name with DEL char must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_empty() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "");
    assert!(result.is_err(), "empty branch name must be rejected");
}

#[test]
fn test_create_branch_rejects_dot_segment() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat/.hidden");
    assert!(
        result.is_err(),
        "branch with segment starting with '.' must be rejected"
    );
}

#[test]
fn test_create_branch_rejects_trailing_dot_segment() {
    let tmp = std::env::temp_dir();
    let result = branch::create_branch(&tmp, "feat/bad./next");
    assert!(
        result.is_err(),
        "branch with segment ending with '.' must be rejected"
    );
}
