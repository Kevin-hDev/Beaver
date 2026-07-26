use super::*;
use tempfile::TempDir;

fn openclaw_spec(home: &Path) -> SourceSpec {
    source_specs_with(home, &home.join(".config"), &home.join(".kimi-code"))
        .into_iter()
        .find(|spec| spec.id == "openclaw")
        .unwrap()
}

#[test]
fn opencode_and_kimi_use_explicit_roots() {
    let temp = TempDir::new().unwrap();
    let xdg = temp.path().join("xdg");
    let kimi = temp.path().join("kimi-home");
    let specs = source_specs_with(temp.path(), &xdg, &kimi);

    let opencode = specs.iter().find(|spec| spec.id == "opencode").unwrap();
    let kimi_spec = specs.iter().find(|spec| spec.id == "kimi").unwrap();

    assert_eq!(opencode.skill_roots, vec![xdg.join("opencode/skills")]);
    assert!(kimi_spec.skill_roots.contains(&kimi.join("skills")));
}

#[test]
fn openclaw_uses_configured_workspace_from_fake_home() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".openclaw");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("openclaw.json"),
        r#"{"agents":{"defaults":{"workspace":"~/custom-claw"}}}"#,
    )
    .unwrap();

    let spec = openclaw_spec(temp.path());

    assert!(spec
        .skill_roots
        .contains(&temp.path().join("custom-claw/skills")));
}

#[test]
fn openclaw_prefers_existing_standard_document() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".openclaw");
    let standard = root.join("workspace");
    let custom = temp.path().join("custom-claw");
    std::fs::create_dir_all(&standard).unwrap();
    std::fs::create_dir_all(&custom).unwrap();
    std::fs::write(standard.join("AGENTS.md"), "standard").unwrap();
    std::fs::write(custom.join("AGENTS.md"), "custom").unwrap();
    std::fs::write(
        root.join("openclaw.json"),
        format!(
            r#"{{"agents":{{"defaults":{{"workspace":"{}"}}}}}}"#,
            custom.display()
        ),
    )
    .unwrap();

    let spec = openclaw_spec(temp.path());

    assert_eq!(spec.documents[0].path, standard.join("AGENTS.md"));
}

#[test]
fn relative_openclaw_workspace_is_rejected() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".openclaw");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("openclaw.json"),
        r#"{"agents":{"defaults":{"workspace":"../secrets"}}}"#,
    )
    .unwrap();

    let spec = openclaw_spec(temp.path());

    assert!(!spec
        .detection_roots
        .iter()
        .any(|path| path.ends_with("secrets")));
}
