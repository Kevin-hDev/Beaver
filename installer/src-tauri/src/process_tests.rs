use super::process::executable_is_running;

#[test]
fn detects_the_exact_running_executable_path() {
    let current = std::env::current_exe().expect("current executable");
    assert_eq!(executable_is_running(&current), Ok(true));
}
