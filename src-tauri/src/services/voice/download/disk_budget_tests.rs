use super::disk_budget::required_bytes;

#[test]
fn existing_durable_archive_bytes_are_not_reserved_twice() {
    assert_eq!(
        required_bytes(100, 200, 40),
        Some(60 + 200 + 64 * 1024 * 1024)
    );
}
