use super::clock::{reached, ticks_at};
use std::time::{Duration, Instant};

#[test]
fn monotonic_deadline_reaches_without_wall_clock() {
    let deadline = ticks_at(Instant::now() + Duration::from_secs(1)).expect("deadline ticks");

    assert!(!reached(deadline).expect("before deadline"));
    let limit = Instant::now() + Duration::from_secs(3);
    while !reached(deadline).expect("deadline check") && Instant::now() < limit {
        std::thread::yield_now();
    }
    assert!(reached(deadline).expect("after deadline"));
}

#[test]
fn zero_is_never_a_valid_monotonic_deadline() {
    assert!(reached(0).is_err());
}
