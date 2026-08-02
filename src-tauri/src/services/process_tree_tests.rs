use super::*;
use std::process::Command;
use std::time::Duration;

#[test]
fn terminate_reaps_child_without_three_second_delay() {
    let mut command = Command::new("/bin/sleep");
    command.arg("30");
    configure(&mut command);
    let mut child = command.spawn().unwrap();
    let started = std::time::Instant::now();

    terminate(&mut child, ProcessKind::ForecastRuntime);

    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(child.try_wait().unwrap().is_some());
}
