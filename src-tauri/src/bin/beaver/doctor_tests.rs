use super::*;

fn check<'a>(results: &'a [CheckResult], label: &str) -> &'a CheckResult {
    results
        .iter()
        .find(|result| result.label_en.contains(label))
        .expect("check exists")
}

#[test]
fn detecte_config_corrompue() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    std::fs::write(directory.path().join("config.json"), b"{ pas du json").expect("broken config");
    assert!(!check(&run_checks(directory.path()), "config").passed);
}

#[test]
fn config_absente_est_saine() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    assert!(check(&run_checks(directory.path()), "config").passed);
}

#[test]
fn detecte_telechargement_interrompu() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    let bundle = directory.path().join("ollama-bundle");
    std::fs::create_dir(&bundle).expect("bundle directory");
    std::fs::write(bundle.join("ollama-bundle-receipt.tmp"), b"partial")
        .expect("temporary receipt");
    assert!(!check(&run_checks(directory.path()), "download").passed);
}

#[test]
fn detecte_recu_bundle_corrompu() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    let bundle = directory.path().join("ollama-bundle");
    std::fs::create_dir(&bundle).expect("bundle directory");
    std::fs::write(bundle.join("ollama-bundle-receipt.json"), b"{").expect("broken receipt");
    std::fs::write(bundle.join("ollama"), b"").expect("empty executable");
    assert!(!check(&run_checks(directory.path()), "bundle").passed);
}

#[test]
fn detecte_vault_manquant_avec_fournisseurs() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    std::fs::write(directory.path().join("configured-providers.json"), b"{}")
        .expect("provider registry");
    assert!(!check(&run_checks(directory.path()), "vault").passed);
}

#[test]
fn detecte_temporaire_racine() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    std::fs::write(directory.path().join("orphan.tmp"), b"x").expect("temporary file");
    assert!(!check(&run_checks(directory.path()), "temporary").passed);
}

#[test]
fn detecte_dossier_de_donnees_absent() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    assert!(!check(&run_checks(&directory.path().join("missing")), "Data").passed);
}

#[test]
fn detecte_journal_app_hors_borne() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    let logs = directory.path().join("logs");
    std::fs::create_dir(&logs).expect("logs");
    let file = std::fs::File::create(logs.join("beaver.log")).expect("app log");
    file.set_len(cl_go_dash_lib::cli_support::APP_LOG_MAX_BYTES * 2 + 1)
        .expect("oversized app log");
    assert!(!check(&run_checks(directory.path()), "log").passed);
}

#[test]
fn compte_la_derniere_ligne_sans_saut_final() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    let logs = directory.path().join("logs");
    std::fs::create_dir(&logs).expect("logs");
    let mut contents =
        "x\n".repeat(cl_go_dash_lib::cli_support::WAKEUP_LOG_MAX_LINES.saturating_mul(2));
    contents.push('x');
    std::fs::write(logs.join("wakeups.jsonl"), contents).expect("wakeup log");
    assert!(!check(&run_checks(directory.path()), "log").passed);
}

#[test]
fn detecte_staging_ollama_orphelin() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    std::fs::create_dir(directory.path().join("ollama-bundle-update-staging")).expect("staging");
    assert!(!check(&run_checks(directory.path()), "staging").passed);
}

#[test]
fn tout_vert_sur_dossier_sain() {
    let directory = tempfile::TempDir::new().expect("temporary directory");
    std::fs::write(directory.path().join("config.json"), b"{}").expect("config");
    std::fs::write(directory.path().join("secrets.enc"), b"x").expect("vault");
    assert!(run_checks(directory.path())
        .iter()
        .filter(|result| !result.label_en.contains("disk"))
        .all(|result| result.passed));
}
