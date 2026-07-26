use super::init_base_structure;

const LEGACY_MEMORY_PATHS: &[&str] = &[
    "memory/archive",
    "memory/episodes",
    "memory/hypotheses",
    "memory/knowledge",
    "memory/procedures",
    "memory/explorer-log.yaml",
];

#[test]
fn fresh_install_only_creates_personality_memory() {
    let root = tempfile::tempdir().unwrap();

    init_base_structure(root.path()).unwrap();

    assert!(root.path().join("memory/core").is_dir());
    assert!(!root.path().join("memory/global").exists());
    assert!(!root.path().join("memory/projects").exists());
    for path in LEGACY_MEMORY_PATHS {
        assert!(!root.path().join(path).exists(), "{path}");
    }
}

#[test]
fn existing_legacy_memory_is_preserved() {
    let root = tempfile::tempdir().unwrap();
    let legacy = root.path().join("memory/knowledge");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("INDEX.md"), "contenu existant").unwrap();

    init_base_structure(root.path()).unwrap();

    assert_eq!(
        std::fs::read_to_string(legacy.join("INDEX.md")).unwrap(),
        "contenu existant"
    );
}
