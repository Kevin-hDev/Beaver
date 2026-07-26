use super::*;

#[test]
fn classify_memory_paths() {
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/memory/core/identity.md"),
        Some(EVENT_PERSONALITY)
    );
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/memory/core"),
        Some(EVENT_PERSONALITY)
    );
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/memory/core/.identity.md.tmp"),
        Some(EVENT_PERSONALITY)
    );
}

#[test]
fn classify_config_files() {
    for path in [
        "/Users/kevin/.local/share/cl-go-dash/config.json",
        "/Users/kevin/.local/share/cl-go-dash/favorite-models.json",
        "/Users/kevin/.local/share/cl-go-dash/agent-settings.json",
    ] {
        assert_eq!(classify_path(path), Some(EVENT_CONFIG));
    }
}

#[test]
fn classify_domain_files() {
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/logs/wakeups.jsonl"),
        Some(EVENT_LOGS)
    );
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/inbox/idea-discovery.md"),
        Some(EVENT_PERSONALITY)
    );
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/mcp-connectors.json"),
        Some(EVENT_CONNECTORS)
    );
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/configured-providers.json"),
        Some(EVENT_PROVIDERS)
    );
}

#[test]
fn classify_skills_paths() {
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/skills"),
        Some(EVENT_SKILLS)
    );
    assert_eq!(
        classify_path("/Users/kevin/.local/share/cl-go-dash/skills/web-search/skill.md"),
        Some(EVENT_SKILLS)
    );
}

#[test]
fn classify_windows_backslash_paths() {
    assert_eq!(
        classify_path("C:\\Users\\kevin\\AppData\\Local\\cl-go-dash\\memory\\core\\identity.md"),
        Some(EVENT_PERSONALITY)
    );
    assert_eq!(
        classify_path("C:\\Users\\kevin\\AppData\\Local\\cl-go-dash\\inbox\\idea.md"),
        Some(EVENT_PERSONALITY)
    );
}

#[test]
fn unknown_paths_are_ignored() {
    for path in [
        "/Users/kevin/.local/share/cl-go-dash/agent-sessions/abc.json",
        "/Users/kevin/.local/share/cl-go-dash/.DS_Store",
    ] {
        assert_eq!(classify_path(path), None);
    }
}

#[test]
fn dedup_emits_unique_events() {
    let paths = [
        "a/memory/core/identity.md",
        "a/memory/core/principles.md",
        "a/config.json",
    ];
    let mut emitted: HashSet<&str> = HashSet::new();
    for path in paths {
        if let Some(event) = classify_path(path) {
            emitted.insert(event);
        }
    }
    assert!(emitted.contains(EVENT_PERSONALITY));
    assert!(emitted.contains(EVENT_CONFIG));
    assert_eq!(emitted.len(), 2);
}

#[test]
fn normalize_path_joins_with_slash() {
    let normalized = normalize_path(&PathBuf::from("/Users/kevin/test/file.md"));
    assert!(normalized.contains("kevin"));
    assert!(normalized.contains("test"));
    assert!(normalized.contains("file.md"));
}
