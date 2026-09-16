use super::disk_budget::{missing_bytes, required_bytes};

#[test]
fn existing_durable_archive_bytes_are_not_reserved_twice() {
    assert_eq!(
        required_bytes(100, 200, 40),
        Some(60 + 200 + 64 * 1024 * 1024)
    );
}

#[test]
fn shortfall_reports_the_exact_missing_bytes() {
    assert_eq!(missing_bytes(1_000, 400), Some(600));
    assert_eq!(missing_bytes(1_000, 1_000), None);
}
