use super::{
    sha256_hex, sync_default_skills_from, ManagedSkillUpgrade, ManagedSkillUpgradeKind,
    MANAGED_SKILL_UPGRADES, MAX_SKILL_MANIFEST_BYTES,
};

fn write_skill(root: &std::path::Path, name: &str, content: &[u8]) {
    let bundle = root.join(name);
    std::fs::create_dir_all(&bundle).unwrap();
    std::fs::write(bundle.join("SKILL.md"), content).unwrap();
}

fn upgrade_for<'a>(name: &'a str, hash: &'a str) -> ManagedSkillUpgrade<'a> {
    ManagedSkillUpgrade {
        name,
        legacy_manifest_sha256: hash,
        kind: ManagedSkillUpgradeKind::FullBundle,
    }
}

#[test]
fn upgrades_an_untouched_forecast_stub_with_the_full_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    write_skill(&resources, "forecasting", b"current");
    write_skill(&installed, "forecasting", b"official-stub");
    std::fs::create_dir_all(resources.join("forecasting/references")).unwrap();
    std::fs::write(
        resources.join("forecasting/references/method.md"),
        b"forecast method",
    )
    .unwrap();
    std::fs::write(installed.join("forecasting/user-note.md"), b"keep").unwrap();
    let legacy_hash = sha256_hex(b"official-stub");
    let upgrades = [upgrade_for("forecasting", &legacy_hash)];

    sync_default_skills_from(&resources, &installed, &upgrades).unwrap();

    assert_eq!(
        std::fs::read(installed.join("forecasting/SKILL.md")).unwrap(),
        b"current"
    );
    assert_eq!(
        std::fs::read(installed.join("forecasting/references/method.md")).unwrap(),
        b"forecast method"
    );
    assert_eq!(
        std::fs::read(installed.join("forecasting/user-note.md")).unwrap(),
        b"keep"
    );
}

#[test]
fn preserves_a_personalized_forecast_bundle() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    write_skill(&resources, "forecasting", b"current");
    write_skill(&installed, "forecasting", b"personalized");
    let legacy_hash = sha256_hex(b"official-stub");
    let upgrades = [upgrade_for("forecasting", &legacy_hash)];

    sync_default_skills_from(&resources, &installed, &upgrades).unwrap();

    assert_eq!(
        std::fs::read(installed.join("forecasting/SKILL.md")).unwrap(),
        b"personalized"
    );
}

#[test]
fn keeps_the_stub_manifest_when_a_full_bundle_is_invalid() {
    let temp = tempfile::tempdir().unwrap();
    let resources = temp.path().join("resources");
    let installed = temp.path().join("installed");
    write_skill(
        &resources,
        "forecasting",
        &vec![b'x'; MAX_SKILL_MANIFEST_BYTES as usize + 1],
    );
    write_skill(&installed, "forecasting", b"official-stub");
    std::fs::create_dir_all(resources.join("forecasting/references")).unwrap();
    std::fs::write(
        resources.join("forecasting/references/method.md"),
        b"method",
    )
    .unwrap();
    let legacy_hash = sha256_hex(b"official-stub");
    let upgrades = [upgrade_for("forecasting", &legacy_hash)];

    assert!(sync_default_skills_from(&resources, &installed, &upgrades).is_err());
    assert_eq!(
        std::fs::read(installed.join("forecasting/SKILL.md")).unwrap(),
        b"official-stub"
    );
}

#[test]
fn bundled_forecast_skills_are_complete_distinct_and_managed() {
    let forecasting = include_str!("../default-skills/forecasting/SKILL.md");
    let router = include_str!("../default-skills/forecast-model-router/SKILL.md");
    let ranking = include_str!(
        "../default-skills/forecast-model-router/references/ranking-and-tournaments.md"
    );
    let chronos =
        include_str!("../default-skills/forecast-model-router/references/family-chronos-bolt.md");

    assert!(forecasting.contains("# Expert Forecasting"));
    assert!(forecasting.contains("forecast_data_audit"));
    assert!(router.contains("# Routeur des modèles Forecast"));
    assert!(router.contains("forecast_models"));
    assert!(!forecasting.contains("[TODO:"));
    assert!(!router.contains("[TODO:"));
    assert!(!forecasting.contains("DOCX skill"));
    assert!(!router.contains("PDF skill"));
    assert!(router.contains("En qualité maximale"));
    assert!(router.contains("le moins coûteux uniquement"));
    assert!(!router.contains("Tu testes d'abord une petite variante compatible"));
    assert!(!ranking.contains("Tu commences par une variante compacte"));
    assert!(chronos.contains("teste Base dès le premier tour"));
    assert_ne!(forecasting, router);
    for name in ["forecasting", "forecast-model-router"] {
        assert!(MANAGED_SKILL_UPGRADES.iter().any(|upgrade| {
            upgrade.name == name && upgrade.kind == ManagedSkillUpgradeKind::FullBundle
        }));
    }
}
