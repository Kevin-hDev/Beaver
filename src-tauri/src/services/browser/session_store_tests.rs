use super::{session_model::SessionModel, session_store};
use std::fs;
use zeroize::Zeroizing;

fn key() -> Zeroizing<Vec<u8>> {
    Zeroizing::new(vec![7_u8; 32])
}

#[test]
fn encrypted_round_trip_never_exposes_the_url() {
    let temp = tempfile::tempdir().unwrap();
    let mut model = SessionModel::new("00000000000000000000000000000001".into()).unwrap();
    model
        .navigate(
            "00000000000000000000000000000001",
            "https://example.com/private-path",
        )
        .unwrap();

    session_store::save_at(
        temp.path(),
        "550e8400-e29b-41d4-a716-446655440000",
        &key(),
        &model,
    )
    .unwrap();
    let path = temp.path().join("550e8400-e29b-41d4-a716-446655440000.enc");
    let raw = fs::read(&path).unwrap();
    assert!(!String::from_utf8_lossy(&raw).contains("private-path"));

    let restored =
        session_store::load_at(temp.path(), "550e8400-e29b-41d4-a716-446655440000", &key())
            .unwrap()
            .unwrap();
    assert_eq!(restored.state().tabs[0].url, model.state().tabs[0].url);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn tampering_fails_closed() {
    let temp = tempfile::tempdir().unwrap();
    let model = SessionModel::new("00000000000000000000000000000001".into()).unwrap();
    let session_id = "550e8400-e29b-41d4-a716-446655440000";
    session_store::save_at(temp.path(), session_id, &key(), &model).unwrap();
    let path = temp.path().join(format!("{session_id}.enc"));
    let mut raw = fs::read(&path).unwrap();
    let last = raw.len() - 2;
    raw[last] ^= 1;
    fs::write(&path, raw).unwrap();

    assert!(session_store::load_at(temp.path(), session_id, &key()).is_err());
}

#[test]
fn rejects_invalid_identifiers_and_oversized_files() {
    let temp = tempfile::tempdir().unwrap();
    assert!(session_store::load_at(temp.path(), "../escape", &key()).is_err());
    let session_id = "550e8400-e29b-41d4-a716-446655440000";
    fs::write(
        temp.path().join(format!("{session_id}.enc")),
        vec![0_u8; session_store::MAX_SESSION_FILE_BYTES + 1],
    )
    .unwrap();
    assert!(session_store::load_at(temp.path(), session_id, &key()).is_err());
}
