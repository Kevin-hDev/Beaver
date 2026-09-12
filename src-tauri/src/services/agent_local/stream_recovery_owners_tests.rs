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

#[test]
fn registry_refuses_an_owner_beyond_its_global_limit() {
    let mut owners = std::collections::HashMap::new();
    for index in 0..MAX_STREAM_RECOVERY_LOGS {
        insert_owner(
            &mut owners,
            (format!("session-{index}"), format!("request-{index}")),
            index as u64,
        )
        .unwrap();
    }

    assert!(insert_owner(&mut owners, ("extra".into(), "extra".into()), u64::MAX).is_err());
}
