use std::collections::HashSet;

use super::super::conversation_admission;
use super::support::ERROR;

#[test]
fn identifier_allocation_retries_collisions_and_reserves_each_result() {
    let collision = "00000000-0000-4000-8000-000000000001".to_string();
    let mut used = HashSet::from([collision.clone()]);
    let candidates = [
        collision,
        "00000000-0000-4000-8000-000000000002".into(),
        "00000000-0000-4000-8000-000000000003".into(),
        "00000000-0000-4000-8000-000000000004".into(),
    ];
    let mut candidates = candidates.into_iter();

    let ids = conversation_admission::allocate_ids_for_test(&mut used, || {
        candidates.next().expect("bounded candidate")
    })
    .expect("collision retried");

    assert_eq!(ids.0, "00000000-0000-4000-8000-000000000002");
    assert_eq!(ids.1, "00000000-0000-4000-8000-000000000003");
    assert_eq!(ids.2, "00000000-0000-4000-8000-000000000004");
    assert_eq!(used.len(), 4);
}

#[test]
fn identifier_allocation_fails_closed_after_bounded_retries() {
    let collision = "00000000-0000-4000-8000-000000000001".to_string();
    let mut used = HashSet::from([collision.clone()]);
    let error = conversation_admission::allocate_ids_for_test(&mut used, || collision.clone())
        .expect_err("repeated collision must stop");

    assert_eq!(error.to_string(), ERROR);
    assert_eq!(used.len(), 1);
}
