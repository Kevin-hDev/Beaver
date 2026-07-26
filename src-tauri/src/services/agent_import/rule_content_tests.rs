use super::*;
use crate::services::agent_import::models::{ImportItem, ImportItemKind};
use tempfile::TempDir;

fn rule(path: PathBuf) -> DiscoveredItem {
    DiscoveredItem {
        public: ImportItem {
            id: "claude:rule:test".into(),
            name: "rule.md".into(),
            description: String::new(),
            source_id: "claude".into(),
            source_name: "Claude Code".into(),
            kind: ImportItemKind::Rule,
            selected: true,
            available: true,
            update_available: false,
        },
        path,
        bundle_root: None,
    }
}

#[test]
fn runtime_rule_read_is_bounded() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("rule.md");
    std::fs::write(&path, vec![b'x'; MAX_INSTRUCTION_BYTES as usize + 1]).unwrap();
    let roots = vec![root.path().canonicalize().unwrap()];

    assert!(read_rule_content(&rule(path), &roots).is_none());
}

#[test]
fn runtime_rule_read_stays_inside_allowed_root() {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let path = outside.path().join("rule.md");
    std::fs::write(&path, "outside").unwrap();
    let roots = vec![root.path().canonicalize().unwrap()];

    assert!(read_rule_content(&rule(path), &roots).is_none());
}

#[cfg(unix)]
#[test]
fn swapped_rule_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let path = root.path().join("rule.md");
    let secret = outside.path().join("secret.md");
    std::fs::write(&path, "safe").unwrap();
    let discovered = rule(path.canonicalize().unwrap());
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&secret, "secret").unwrap();
    symlink(&secret, &path).unwrap();
    let roots = vec![root.path().canonicalize().unwrap()];

    assert!(read_rule_content(&discovered, &roots).is_none());
}
