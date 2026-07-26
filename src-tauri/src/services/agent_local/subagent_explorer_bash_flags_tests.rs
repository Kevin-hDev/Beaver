use super::subagent_explorer_bash::validate;

#[test]
fn rejects_options_that_follow_links_recurse_or_write() {
    let root = tempfile::tempdir().expect("root");

    for command in [
        "ls -L .",
        "ls -R .",
        "ls -LR .",
        "du -L .",
        "tree -L 3 -l .",
        "git diff --output=patch.txt",
        "git log --output=history.txt",
        "git show --output=commit.txt HEAD",
    ] {
        assert!(validate(command, root.path()).is_err(), "commande acceptée: {command}");
    }
}

#[test]
fn keeps_supported_informational_options() {
    let root = tempfile::tempdir().expect("root");

    for command in [
        "ls -lah .",
        "wc -l Cargo.toml",
        "du -sh .",
        "df -h .",
        "git status --short",
        "git diff --stat",
        "git log -5",
        "git show --stat HEAD",
    ] {
        assert!(validate(command, root.path()).is_ok(), "commande refusée: {command}");
    }
}
