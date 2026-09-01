use super::exit_wait::{reap_child, wait_for_exit_code, ExitPoll};
use std::time::{Duration, Instant};

#[test]
fn delayed_exit_code_is_observed_before_the_short_budget_expires() {
    let mut probes = 0;

    let code = wait_for_exit_code(|| {
        probes += 1;
        if probes < 3 {
            ExitPoll::Running
        } else {
            ExitPoll::Exited(Some(3))
        }
    });

    assert_eq!(code, Some(3));
    assert_eq!(probes, 3);
}

#[test]
fn missing_exit_code_stays_unknown_after_the_real_budget() {
    let started = Instant::now();

    let code = wait_for_exit_code(|| ExitPoll::Running);

    assert_eq!(code, None);
    assert!(started.elapsed() >= Duration::from_millis(150));
}

#[test]
fn child_reap_returns_after_its_bounded_budget() {
    let started = Instant::now();

    let code = reap_child(|| ExitPoll::Running);

    assert_eq!(code, None);
    assert!(started.elapsed() >= Duration::from_millis(1_800));
    assert!(started.elapsed() < Duration::from_secs(3));
}
