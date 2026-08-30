use std::time::Duration;

pub fn delay(attempt: u8) -> Duration {
    match attempt {
        0 => Duration::from_millis(250),
        _ => Duration::from_secs(1),
    }
}
