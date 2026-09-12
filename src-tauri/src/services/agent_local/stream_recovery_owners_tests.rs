use super::stream_recovery_owners::*;

#[test]
fn lease_exclusively_owns_one_request_until_drop() {
    let session = uuid::Uuid::new_v4().to_string();
    let request = uuid::Uuid::new_v4().to_string();
    let lease = claim(&session, &request).expect("first owner");
    assert!(is_live(&session, &request));
    assert!(claim(&session, &request).is_err());
    drop(lease);
    assert!(!is_live(&session, &request));
    drop(claim(&session, &request).expect("reclaimed"));
}
