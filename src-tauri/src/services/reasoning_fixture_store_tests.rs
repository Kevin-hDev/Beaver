use super::*;

#[test]
fn store_evicts_the_oldest_report_without_following_links() {
    let temporary = tempfile::tempdir().unwrap();
    let directory = temporary.path().join("reports");
    let session = "11111111-1111-4111-8111-111111111111";
    let date = chrono::NaiveDate::from_ymd_opt(2026, 8, 26).unwrap();
    for index in 0..MAX_REPORTS {
        write_at(
            &directory,
            session,
            &derive_fixture_id("ollama", &format!("fixture-{index}"), "test", date).unwrap(),
            b"{}",
        )
        .unwrap();
    }
    write_at(
        &directory,
        session,
        &derive_fixture_id("ollama", "fixture-overflow", "test", date).unwrap(),
        b"{}",
    )
    .unwrap();
    assert_eq!(
        valid_reports(&directory.join(session)).unwrap().len(),
        MAX_REPORTS
    );
}

#[test]
fn unrelated_regular_file_does_not_block_pruning() {
    let temporary = tempfile::tempdir().unwrap();
    let directory = temporary.path().join("reports/session");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join(".DS_Store"), b"metadata").unwrap();

    assert!(valid_reports(&directory).unwrap().is_empty());
}

#[test]
fn rejects_noncanonical_report_names() {
    assert!(!is_report_name("report.json"));
    assert!(!is_report_name("../report.json"));
    assert!(!is_report_name("11111111-1111-4111-8111-111111111111.json"));
}

#[test]
fn derives_a_bounded_canonical_report_name() {
    let id = derive_fixture_id(
        "xai-oauth",
        "Grok 4.6/preview",
        "eu-west-1",
        chrono::NaiveDate::from_ymd_opt(2026, 8, 26).unwrap(),
    )
    .unwrap();
    assert_eq!(id, "xai-oauth-grok-4-6-preview-eu-west-1-2026-08-26");
    assert!(validate_fixture_id(&id).is_ok());
    assert!(
        derive_fixture_id("xai", "model", "../escape", chrono::Utc::now().date_naive()).is_err()
    );
}
