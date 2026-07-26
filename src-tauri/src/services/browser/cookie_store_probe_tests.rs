use super::cookie_store_probe::{
    cookie_store_hides_probe, MAX_COOKIE_STORE_BYTES, PROBE_VALUE_BYTES,
};
use std::fs;

fn probe() -> [u8; PROBE_VALUE_BYTES] {
    [b'a'; PROBE_VALUE_BYTES]
}

fn profile_with_store(contents: &[u8]) -> tempfile::TempDir {
    let profile = tempfile::tempdir().expect("temporary profile");
    let default = profile.path().join("Default");
    fs::create_dir(&default).expect("default profile");
    fs::write(default.join("Cookies"), contents).expect("cookie store");
    profile
}

#[test]
fn accepts_store_without_plaintext_probe() {
    let profile = profile_with_store(b"encrypted cookie bytes");

    assert_eq!(cookie_store_hides_probe(profile.path(), &probe()), Ok(true));
}

#[test]
fn detects_probe_split_across_read_chunks() {
    let mut contents = vec![b'x'; 64 * 1024 - 17];
    contents.extend_from_slice(&probe());
    let profile = profile_with_store(&contents);

    assert_eq!(
        cookie_store_hides_probe(profile.path(), &probe()),
        Ok(false)
    );
}

#[test]
fn detects_probe_in_cookie_journal() {
    let profile = profile_with_store(b"database");
    fs::write(profile.path().join("Default/Cookies-journal"), probe()).expect("cookie journal");

    assert_eq!(
        cookie_store_hides_probe(profile.path(), &probe()),
        Ok(false)
    );
}

#[test]
fn fails_closed_when_cookie_database_is_missing() {
    let profile = tempfile::tempdir().expect("temporary profile");

    assert_eq!(cookie_store_hides_probe(profile.path(), &probe()), Err(()));
}

#[test]
fn fails_closed_when_store_exceeds_the_fixed_limit() {
    let profile = profile_with_store(b"database");
    let store = fs::OpenOptions::new()
        .write(true)
        .open(profile.path().join("Default/Cookies"))
        .expect("open cookie store");
    store
        .set_len(MAX_COOKIE_STORE_BYTES + 1)
        .expect("sparse oversized store");

    assert_eq!(cookie_store_hides_probe(profile.path(), &probe()), Err(()));
}
