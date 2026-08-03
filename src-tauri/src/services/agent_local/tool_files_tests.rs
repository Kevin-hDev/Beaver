use crate::services::agent_local::tool_files::{
    edit_file, read_file, write_file, write_file_in_roots, DEFAULT_LIMIT,
};
use crate::services::agent_local::tool_result_contract::ToolResultStatus;
// MAX_LIMIT est 50_000 — on le réimporte pour les tests de borne
const MAX_LIMIT: usize = 50_000;
use std::io::Write;
use tempfile::NamedTempFile;

fn make_temp_file(lines: &[&str]) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    for line in lines {
        writeln!(f, "{line}").expect("write");
    }
    f
}

#[tokio::test]
async fn read_file_full() {
    let lines: Vec<&str> = (1..=10).map(|_| "hello world").collect();
    let f = make_temp_file(&lines);
    let working_dir = f.path().parent().unwrap();
    let result = read_file(f.path().to_str().unwrap(), working_dir, 0, DEFAULT_LIMIT).await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // 10 lignes numérotées de 1 à 10
    for i in 1..=10usize {
        assert!(
            result.content.contains(&format!("{i}\thello world")),
            "ligne {i} absente"
        );
    }
    assert!(
        !result.content.contains("restante"),
        "ne doit pas avoir de message de continuation"
    );
}

#[tokio::test]
async fn read_file_offset_limit() {
    let lines: Vec<String> = (1..=20).map(|i| format!("line{i}")).collect();
    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let f = make_temp_file(&lines_ref);
    let working_dir = f.path().parent().unwrap();
    // Lire lignes 6 à 10 (offset=5, limit=5)
    let result = read_file(f.path().to_str().unwrap(), working_dir, 5, 5).await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // Doit contenir lignes 6-10 (numérotées 6..10)
    for i in 6..=10usize {
        assert!(
            result.content.contains(&format!("{i}\tline{i}")),
            "ligne {i} absente"
        );
    }
    // Ne doit pas contenir line1..5 ni line11..20
    assert!(
        !result.content.contains("1\tline1"),
        "ne doit pas contenir line1"
    );
    assert!(
        !result.content.contains("11\tline11"),
        "ne doit pas contenir line11"
    );
    // Message de continuation : 10 lignes restantes, offset=10
    assert!(
        result.content.contains("offset=10"),
        "doit indiquer offset=10 pour la suite"
    );
    assert!(
        result.content.contains("10 ligne(s) restante(s)"),
        "doit indiquer 10 lignes restantes"
    );
    assert_eq!(result.status, ToolResultStatus::Partial);
    assert!(result.truncated);
}

#[tokio::test]
async fn read_file_offset_beyond_end() {
    let lines = vec!["a", "b", "c"];
    let f = make_temp_file(&lines);
    let working_dir = f.path().parent().unwrap();
    // offset=100 dépasse la fin (3 lignes)
    let result = read_file(f.path().to_str().unwrap(), working_dir, 100, DEFAULT_LIMIT).await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // Contenu vide (aucune ligne)
    assert!(
        result.content.trim().is_empty(),
        "le contenu doit être vide pour un offset au-delà de la fin"
    );
}

#[tokio::test]
async fn read_file_default_limit() {
    // Crée un fichier de 2500 lignes
    let lines: Vec<String> = (1..=2500).map(|i| format!("row{i}")).collect();
    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let f = make_temp_file(&lines_ref);
    let working_dir = f.path().parent().unwrap();
    let result = read_file(f.path().to_str().unwrap(), working_dir, 0, DEFAULT_LIMIT).await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // DEFAULT_LIMIT = 2000, doit s'arrêter à la ligne 2000
    assert!(
        result.content.contains("2000\trow2000"),
        "doit contenir la ligne 2000"
    );
    assert!(
        !result.content.contains("2001\trow2001"),
        "ne doit pas contenir la ligne 2001"
    );
    // Message de continuation : 500 lignes restantes
    assert!(
        result.content.contains("500 ligne(s) restante(s)"),
        "doit indiquer 500 lignes restantes"
    );
    assert!(
        result.content.contains("offset=2000"),
        "doit indiquer offset=2000 pour la suite"
    );
    assert_eq!(result.status, ToolResultStatus::Partial);
    assert!(result.truncated);
}

#[tokio::test]
async fn read_file_limit_capped_at_max() {
    // Un fichier de 100 lignes, on demande limit = usize::MAX (overflow potentiel)
    let lines: Vec<String> = (1..=100).map(|i| format!("row{i}")).collect();
    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let f = make_temp_file(&lines_ref);
    let working_dir = f.path().parent().unwrap();
    let result = read_file(
        f.path().to_str().unwrap(),
        working_dir,
        0,
        usize::MAX, // valeur extrême — doit être bornée à MAX_LIMIT
    )
    .await;
    assert!(!result.is_error, "ne doit pas être une erreur");
    // Toutes les 100 lignes doivent être présentes (100 < MAX_LIMIT)
    assert!(
        result.content.contains("100\trow100"),
        "la ligne 100 doit être présente"
    );
    // Pas de message de continuation (toutes les lignes lues)
    assert!(
        !result.content.contains("restante"),
        "ne doit pas avoir de message de continuation"
    );
}

#[tokio::test]
async fn read_file_limit_overflow_with_large_offset() {
    // Test que saturating_add empêche l'overflow : offset + limit ne doit pas déborder
    let lines: Vec<String> = (1..=10).map(|i| format!("row{i}")).collect();
    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let f = make_temp_file(&lines_ref);
    let working_dir = f.path().parent().unwrap();
    // offset = MAX_LIMIT - 1, limit = MAX_LIMIT → saturating_add bornerait à 2*MAX_LIMIT-1
    // mais le fichier n'a que 10 lignes donc start = min(MAX_LIMIT-1, 10) = 10
    let result = read_file(
        f.path().to_str().unwrap(),
        working_dir,
        MAX_LIMIT - 1,
        MAX_LIMIT,
    )
    .await;
    assert!(!result.is_error, "ne doit pas paniquer ni déborder");
    assert!(
        result.content.trim().is_empty(),
        "offset au-delà de la fin → contenu vide"
    );
}

#[tokio::test]
async fn edit_file_returns_a_structured_start_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("example.txt");
    tokio::fs::write(&path, "première\nseconde\ntroisième")
        .await
        .unwrap();

    let result = edit_file(path.to_str().unwrap(), "seconde", "modifiée", dir.path()).await;

    assert!(!result.is_error);
    assert_eq!(result.start_line(), Some(2));
}

#[tokio::test]
async fn read_file_reports_non_utf8_content_as_unsupported_input() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("binary.bin");
    std::fs::write(&path, [0xff, 0xfe]).unwrap();

    let result = read_file(path.to_str().unwrap(), directory.path(), 0, DEFAULT_LIMIT).await;

    assert_eq!(result.error.unwrap().code.as_ref(), "file_not_utf8");
}

#[tokio::test]
async fn write_file_uses_the_same_effective_roots_for_both_checks() {
    let workspace_root = tempfile::tempdir().expect("workspace");
    let working_dir = workspace_root.path().join("work");
    std::fs::create_dir_all(&working_dir).expect("work");
    let canonical_working = dunce::canonicalize(&working_dir).expect("work root");
    let roots = vec![canonical_working.clone()];

    let result = write_file_in_roots("created.txt", "managed", &working_dir, &roots).await;

    assert!(roots.iter().any(|root| canonical_working.starts_with(root)));
    assert!(!result.is_error, "{}", result.content);
    assert_eq!(std::fs::read_to_string(working_dir.join("created.txt")).unwrap(), "managed");
}

#[tokio::test]
async fn write_file_rejects_a_path_outside_the_effective_roots() {
    let allowed = tempfile::tempdir().expect("allowed");
    let outside = tempfile::tempdir().expect("outside");
    let roots = vec![dunce::canonicalize(allowed.path()).expect("allowed root")];

    let result = write_file_in_roots("blocked.txt", "blocked", outside.path(), &roots).await;

    assert!(result.is_error);
    assert!(!outside.path().join("blocked.txt").exists());
}

#[cfg(unix)]
#[tokio::test]
async fn write_file_rejects_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    let link = dir.path().join("link.txt");
    std::fs::write(&target, "original").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let result = write_file(link.to_str().unwrap(), "changed", dir.path()).await;

    assert!(result.is_error);
    assert_eq!(std::fs::read_to_string(target).unwrap(), "original");
}
