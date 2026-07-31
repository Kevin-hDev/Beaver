use super::test_support::{commit_all, managed};
use super::*;
use tokio_util::sync::CancellationToken;

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn stdout_and_stderr_remain_distinct() {
    let dir = tempfile::tempdir().expect("tempdir");

    let output = execute_shell(
        "printf out; printf error >&2; printf tail",
        dir.path(),
        None,
    )
    .await
    .expect("shell output");

    assert_eq!(output.stdout, "outtail");
    assert_eq!(output.stderr, "error");
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn shell_diff_starts_from_the_dirty_pre_command_content() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repository = git2::Repository::init(dir.path()).expect("repository");
    let file = dir.path().join("tracked.txt");
    std::fs::write(&file, "committed\n").expect("initial file");
    commit_all(&repository);
    std::fs::write(&file, "dirty before command\n").expect("dirty file");

    let output = execute_shell("printf 'after command\\n' > tracked.txt", dir.path(), None)
        .await
        .expect("shell edit");
    let change = output
        .file_changes
        .iter()
        .find(|change| change.path.ends_with("tracked.txt"))
        .expect("tracked change");
    let diff = change.diff.as_ref().expect("content diff");
    let content = diff
        .hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .map(|line| line.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(content.contains("dirty before command"));
    assert!(content.contains("after command"));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn large_output_keeps_a_bounded_preview_and_a_complete_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = execute_shell("yes x | head -c 2000000", dir.path(), None)
        .await
        .expect("large output");
    let relative = output
        .stdout
        .split("[Résultat complet disponible : ")
        .nth(1)
        .and_then(|tail| tail.split(']').next())
        .expect("output path");
    let path = crate::services::paths::data_dir().join(relative);
    let full = tokio::fs::read(&path).await.expect("full output");

    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.len() < 30_000);
    assert_eq!(full.len(), 2_000_000);
    tokio::fs::remove_file(path).await.expect("cleanup output");
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn short_output_does_not_create_a_result_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let output = managed(
        "printf small",
        dir.path(),
        &owner,
        None,
        None,
        CancellationToken::new(),
    )
    .await
    .expect("short output");
    let output_dir = crate::services::paths::data_dir()
        .join("tool-results")
        .join(owner);

    assert_eq!(output.stdout, "small");
    assert!(!output_dir.exists());
}
