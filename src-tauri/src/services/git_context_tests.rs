use super::*;
use std::path::Path;

#[test]
fn test_detect_git_in_repo() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let snap = detect_git(manifest);
    assert!(snap.is_git, "Le repo doit être détecté comme git");
    assert!(
        snap.current_branch.is_some(),
        "La branche doit être détectée"
    );
    assert!(snap.git_root.is_some(), "Le git root doit être trouvé");
}

#[test]
fn test_detect_git_outside_repo() {
    let snap = detect_git(Path::new("/tmp"));
    assert!(!snap.is_git);
    assert!(snap.current_branch.is_none());
    assert!(snap.git_root.is_none());
}

#[test]
fn test_format_git_section_none_if_not_git() {
    assert!(format_git_section(&GitSnapshot::default()).is_none());
}

#[test]
fn test_format_git_section_has_content() {
    let snap = GitSnapshot {
        is_git: true,
        current_branch: Some("main".to_string()),
        default_branch: Some("main".to_string()),
        status_short: Some("M  src/main.rs".to_string()),
        recent_commits: Some("abc1234 test commit".to_string()),
        git_root: None,
    };
    let section = format_git_section(&snap).unwrap();
    assert!(section.contains("Current branch: main"));
    assert!(section.contains("M  src/main.rs"));
    assert!(section.contains("abc1234 test commit"));
}

#[test]
fn test_format_git_section_truncation() {
    let snap = GitSnapshot {
        is_git: true,
        current_branch: Some("main".to_string()),
        default_branch: None,
        status_short: Some("M  ".to_string() + &"a".repeat(3000)),
        recent_commits: None,
        git_root: None,
    };
    let section = format_git_section(&snap).unwrap();
    assert!(
        section.len() <= 2010,
        "Tronqué — got {} chars",
        section.len()
    );
}

#[test]
fn test_no_shell_injection() {
    let snap = detect_git(Path::new("/tmp/foo; rm -rf /"));
    assert!(!snap.is_git);
}
