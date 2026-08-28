use super::*;

#[test]
fn retired_groq_secret_and_scope_are_removed_together() {
    let mut map = HashMap::from([
        ("groq".to_string(), "retired-secret".to_string()),
        (
            "raw:reasoning_scope:groq".to_string(),
            "retired-scope".to_string(),
        ),
        ("openai".to_string(), "active-secret".to_string()),
    ]);

    assert!(purge_retired_provider_entries(&mut map));
    assert!(!map.contains_key("groq"));
    assert!(!map.contains_key("raw:reasoning_scope:groq"));
    assert_eq!(map.get("openai").map(String::as_str), Some("active-secret"));
}

#[test]
fn retired_cleanup_does_not_rewrite_an_unaffected_vault_map() {
    let mut map = HashMap::from([("openai".to_string(), "active-secret".to_string())]);

    assert!(!purge_retired_provider_entries(&mut map));
    assert_eq!(map.len(), 1);
}

#[test]
fn encrypted_vault_backup_is_preserved_until_the_next_successful_start() {
    let root = tempfile::tempdir().unwrap();
    let vault_path = root.path().join("secrets.enc");
    let backup_path = root.path().join("secrets.enc.pre-groq-removal.bak");
    std::fs::write(&vault_path, b"encrypted-before-cleanup").unwrap();

    backup_vault_for_retired_cleanup(&vault_path, &backup_path).unwrap();
    std::fs::write(&vault_path, b"encrypted-after-cleanup").unwrap();
    backup_vault_for_retired_cleanup(&vault_path, &backup_path).unwrap();

    assert_eq!(
        std::fs::read(&backup_path).unwrap(),
        b"encrypted-before-cleanup"
    );
    remove_retired_cleanup_backup(&backup_path).unwrap();
    assert!(!backup_path.exists());
}

#[cfg(unix)]
#[test]
fn retired_cleanup_refuses_a_symbolic_backup_path() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let vault_path = root.path().join("secrets.enc");
    let backup_path = root.path().join("secrets.enc.pre-groq-removal.bak");
    let outside = root.path().join("outside");
    std::fs::write(&vault_path, b"encrypted-before-cleanup").unwrap();
    std::fs::write(&outside, b"do-not-trust").unwrap();
    symlink(&outside, &backup_path).unwrap();

    assert!(backup_vault_for_retired_cleanup(&vault_path, &backup_path).is_err());
}
