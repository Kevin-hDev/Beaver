use super::watcher::parse_gitdir_content;
use std::path::PathBuf;

#[test]
fn parses_absolute_gitdir() {
    let content = "gitdir: /Users/dev/proj/.git/worktrees/feature\n";
    assert_eq!(
        parse_gitdir_content(content),
        Some(PathBuf::from("/Users/dev/proj/.git/worktrees/feature"))
    );
}

#[test]
fn parses_relative_gitdir() {
    let content = "gitdir: ../.git/worktrees/feature";
    assert_eq!(
        parse_gitdir_content(content),
        Some(PathBuf::from("../.git/worktrees/feature"))
    );
}

#[test]
fn trims_whitespace() {
    let content = "  gitdir: /abs/path  \n";
    assert_eq!(
        parse_gitdir_content(content),
        Some(PathBuf::from("/abs/path"))
    );
}

#[test]
fn rejects_invalid_or_empty_content() {
    assert!(parse_gitdir_content("/just/a/path").is_none());
    assert!(parse_gitdir_content("not a gitdir file").is_none());
    assert!(parse_gitdir_content("").is_none());
    assert!(parse_gitdir_content("gitdir: ").is_none());
    assert!(parse_gitdir_content("gitdir:").is_none());
}

#[test]
fn parses_unix_worktree_path() {
    let content = "gitdir: /home/user/repo/.git/worktrees/feat-x";
    let result = parse_gitdir_content(content).expect("gitdir");
    assert!(result.starts_with("/home/user/repo"));
}

#[test]
fn handles_trailing_newline_only() {
    let content = "gitdir: /path/to/.git/worktrees/main\n";
    assert_eq!(
        parse_gitdir_content(content),
        Some(PathBuf::from("/path/to/.git/worktrees/main"))
    );
}
