use std::time::Duration;

// Windows scanners may briefly hold a file between its durable write and atomic
// replacement. This single bounded budget is shared by every durable store.
pub(crate) const INTERVAL: Duration = Duration::from_millis(50);
pub(crate) const TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) fn bounded<T, E, Operation, Retryable, Sleep>(
    mut operation: Operation,
    mut retryable: Retryable,
    mut sleep: Sleep,
) -> Result<T, E>
where
    Operation: FnMut() -> Result<T, E>,
    Retryable: FnMut(&E) -> bool,
    Sleep: FnMut(Duration),
{
    let max_waits = TIMEOUT
        .as_millis()
        .checked_div(INTERVAL.as_millis())
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);
    let mut waits = 0;
    loop {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) if retryable(&error) && waits < max_waits => {
                sleep(INTERVAL);
                waits += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_failures_are_retried_with_one_bounded_authority() {
        let mut attempts = 0;
        let mut waits = Vec::new();
        let result = bounded(
            || {
                attempts += 1;
                if attempts < 3 {
                    Err("busy")
                } else {
                    Ok(())
                }
            },
            |error| *error == "busy",
            |delay| waits.push(delay),
        );

        assert_eq!(result, Ok(()));
        assert_eq!(attempts, 3);
        assert_eq!(waits, [INTERVAL; 2]);
    }

    #[test]
    fn permanent_failure_is_bounded() {
        let mut waits = 0;
        let result = bounded(|| Err::<(), _>("busy"), |_| true, |_| waits += 1);

        assert_eq!(result, Err("busy"));
        assert_eq!(waits, 40);
    }
}
