//! Waiting for a person never consumes or resets the active execution budget.
use super::process_runner::ProcessFailure;
use std::time::{Duration, Instant};

pub(super) fn run<T>(
    remaining: &mut Duration,
    attempt: impl FnMut(Duration) -> Result<T, ProcessFailure>,
    stopped: impl FnMut() -> Result<bool, ProcessFailure>,
) -> Result<T, ProcessFailure> {
    let origin = Instant::now();
    with_clock(remaining, attempt, stopped, || origin.elapsed())
}

fn with_clock<T>(
    remaining: &mut Duration,
    mut attempt: impl FnMut(Duration) -> Result<T, ProcessFailure>,
    mut stopped: impl FnMut() -> Result<bool, ProcessFailure>,
    now: impl Fn() -> Duration,
) -> Result<T, ProcessFailure> {
    loop {
        if remaining.is_zero() {
            return Err(ProcessFailure::Timeout);
        }
        let started = now();
        let result = attempt(*remaining);
        *remaining = remaining.saturating_sub(now().saturating_sub(started));
        if matches!(&result, Err(error) if *error != ProcessFailure::Interrupted) {
            return result;
        }
        let continued = stopped()?;
        if result.is_ok() || !continued {
            return result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn human_wait_neither_spends_nor_refills_the_active_budget() {
        let clock = std::cell::Cell::new(0_u64);
        let mut attempts = 0;
        let mut remaining = Duration::from_millis(10);
        let result = with_clock(
            &mut remaining,
            |budget| {
                attempts += 1;
                if attempts == 1 {
                    clock.set(clock.get() + 3);
                    Err(ProcessFailure::Interrupted)
                } else {
                    assert_eq!(budget, Duration::from_millis(7));
                    clock.set(clock.get() + 2);
                    Ok(())
                }
            },
            || {
                clock.set(clock.get() + 5000);
                Ok(true)
            },
            || Duration::from_millis(clock.get()),
        );
        assert!(result.is_ok());
        assert_eq!(remaining, Duration::from_millis(5));
    }
    #[test]
    fn unconfirmed_stop_never_offers_a_retry() {
        assert_eq!(
            run::<()>(
                &mut Duration::from_secs(1),
                |_| Err(ProcessFailure::StopUnconfirmed),
                || panic!("must not wait for consent while a producer may live")
            ),
            Err(ProcessFailure::StopUnconfirmed)
        );
    }
}
