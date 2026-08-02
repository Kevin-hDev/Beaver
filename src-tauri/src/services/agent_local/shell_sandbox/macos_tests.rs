use super::{policy, read_roots, SANDBOX_EXEC};
use std::process::Command;

#[test]
fn seatbelt_allows_work_inside_the_root_and_blocks_outside_data() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let outside = temp.path().join("outside.txt");
    std::fs::create_dir(&allowed).expect("allowed");
    std::fs::write(&outside, "secret").expect("outside");
    let allowed = dunce::canonicalize(allowed).expect("canonical allowed");
    let outside = dunce::canonicalize(outside).expect("canonical outside");
    let script = format!(
        "printf ok > '{}/inside.txt'; /bin/cat '{}' >/dev/null 2>&1 && exit 91; /bin/sh -c 'printf child'",
        allowed.display(),
        outside.display(),
    );

    let read_roots = read_roots(std::slice::from_ref(&allowed));
    let mut command = Command::new(SANDBOX_EXEC);
    command
        .current_dir(&allowed)
        .args(["-D", &format!("BEAVER_RW_0={}", allowed.display())]);
    for (index, root) in read_roots.iter().enumerate() {
        command.args(["-D", &format!("BEAVER_RO_{index}={}", root.display())]);
    }
    let output = command
        .args(["-p", &policy(1, read_roots.len()), "/bin/sh", "-c", &script])
        .output()
        .expect("sandbox-exec");

    assert!(
        output.status.success(),
        "status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(std::fs::read_to_string(allowed.join("inside.txt")).unwrap(), "ok");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "child");
}
