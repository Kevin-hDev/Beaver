use super::*;

#[test]
fn inventaire_ignore_les_donnees_protegees() {
    let root = tempfile::TempDir::new().expect("temporary directory");
    std::fs::create_dir(root.path().join("agent-sessions")).expect("sessions");
    std::fs::write(root.path().join("agent-sessions/s1.json"), b"x").expect("session");
    std::fs::write(root.path().join("secrets.enc"), b"x").expect("vault");
    std::fs::write(root.path().join("config.json"), b"{}").expect("config");
    assert!(inventory(root.path(), std::time::SystemTime::now())
        .expect("inventory")
        .is_empty());
}

#[test]
fn inventaire_attrape_vieille_rotation_de_log() {
    let root = tempfile::TempDir::new().expect("temporary directory");
    let logs = root.path().join("logs");
    std::fs::create_dir(&logs).expect("logs");
    std::fs::write(logs.join("beaver.log"), b"active").expect("active log");
    let old = logs.join("beaver_2026-06-01.log");
    std::fs::write(&old, b"old").expect("old log");
    let future = std::time::SystemTime::now() + std::time::Duration::from_secs(40 * 86_400);
    let candidates = inventory(root.path(), future).expect("inventory");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].path, old);
}

#[test]
fn inventaire_inclut_temporaire_et_vieux_resultat() {
    let root = tempfile::TempDir::new().expect("temporary directory");
    let bundle = root.path().join("ollama-bundle");
    std::fs::create_dir(&bundle).expect("bundle");
    std::fs::write(bundle.join("download.partial"), b"x").expect("partial");
    let result = root.path().join("tool-results/session");
    std::fs::create_dir_all(&result).expect("result");
    std::fs::write(result.join("full.txt"), b"x").expect("result file");
    let future = std::time::SystemTime::now() + std::time::Duration::from_secs(2 * 86_400);
    let candidates = inventory(root.path(), future).expect("inventory");
    assert_eq!(candidates.len(), 2);
}

#[test]
fn accepte_fichier_reellement_dedans() {
    let root = tempfile::TempDir::new().expect("temporary directory");
    let logs = root.path().join("logs");
    std::fs::create_dir(&logs).expect("logs");
    let file = logs.join("old.log");
    std::fs::write(&file, b"x").expect("log");
    assert!(is_safely_inside(&logs, &file));
}

#[cfg(unix)]
#[test]
fn refuse_lien_symbolique_sortant() {
    use std::os::unix::fs::symlink;

    let root = tempfile::TempDir::new().expect("temporary directory");
    let outside = tempfile::TempDir::new().expect("outside directory");
    let victim = outside.path().join("victim.txt");
    std::fs::write(&victim, b"precious").expect("victim");
    let link = root.path().join("linked");
    symlink(&victim, &link).expect("symlink");
    assert!(!is_safely_inside(root.path(), &link));
}

#[cfg(unix)]
#[test]
fn refuse_dossier_famille_lie_vers_exterieur() {
    use std::os::unix::fs::symlink;

    let root = tempfile::TempDir::new().expect("temporary directory");
    let outside = tempfile::TempDir::new().expect("outside directory");
    let victim = outside.path().join("victim.txt");
    std::fs::write(&victim, b"precious").expect("victim");
    let family = root.path().join("tool-results");
    symlink(outside.path(), &family).expect("symlink");
    assert!(!is_safely_inside(&family, &family.join("victim.txt")));
}

#[cfg(unix)]
#[test]
fn refuse_dossier_famille_lie_vers_conversations() {
    use std::os::unix::fs::symlink;

    let root = tempfile::TempDir::new().expect("temporary directory");
    let sessions = root.path().join("agent-sessions");
    std::fs::create_dir_all(sessions.join("conv1")).expect("conversation");
    let family = root.path().join("tool-results");
    symlink(&sessions, &family).expect("symlink");
    assert!(!is_safely_inside(&family, &family.join("conv1")));
}

#[cfg(unix)]
#[test]
fn inventaire_distingue_vide_et_illisible() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::TempDir::new().expect("temporary directory");
    assert!(inventory(root.path(), std::time::SystemTime::now())
        .expect("empty inventory")
        .is_empty());
    let logs = root.path().join("logs");
    std::fs::create_dir(&logs).expect("logs");
    let mut permissions = std::fs::metadata(&logs).expect("metadata").permissions();
    permissions.set_mode(0o000);
    std::fs::set_permissions(&logs, permissions.clone()).expect("deny permissions");
    let result = inventory(root.path(), std::time::SystemTime::now());
    permissions.set_mode(0o755);
    std::fs::set_permissions(&logs, permissions).expect("restore permissions");
    assert!(result.is_err());
}

#[test]
fn suppression_continue_apres_un_echec() {
    let root = tempfile::TempDir::new().expect("temporary directory");
    let logs = root.path().join("logs");
    std::fs::create_dir(&logs).expect("logs");
    let removable = logs.join("removable.log");
    let missing = logs.join("missing.log");
    std::fs::write(&removable, b"x").expect("removable");
    let candidates = [
        Candidate {
            path: missing.clone(),
            bytes: 1,
            reason_fr: "échec",
            reason_en: "failure",
            family: logs.clone(),
            kind: CandidateKind::File,
        },
        Candidate {
            path: removable.clone(),
            bytes: 1,
            reason_fr: "suppression",
            reason_en: "removal",
            family: logs,
            kind: CandidateKind::File,
        },
    ];
    let summary = cleanup_remove::remove_candidates(root.path(), &candidates);
    assert_eq!(summary.removed, 1);
    assert_eq!(summary.failed, vec![missing]);
    assert_eq!(summary.recovered, 1);
    assert!(!removable.exists());
}
