const CHILD_MARKER: &str = "BEAVER_ASYNC_STACK_CHILD";
const TEST_NAME: &str = "runtime_async::tests::tauri_workers_have_room_for_agent_tools";
const PROBE_BYTES: usize = 3 * 1024 * 1024;

#[test]
fn tauri_workers_have_room_for_agent_tools() {
    if std::env::var_os(CHILD_MARKER).is_none() {
        let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", TEST_NAME, "--nocapture"])
            .env(CHILD_MARKER, "1")
            .status()
            .expect("spawn cold test process");
        assert!(status.success(), "cold child failed with {status}");
        return;
    }

    let runtime = super::build().expect("application runtime");
    let observed = runtime.block_on(async {
        tokio::spawn(async { stack_probe() })
            .await
            .expect("worker must keep running")
    });
    assert_eq!(observed, 2);
}

#[inline(never)]
fn stack_probe() -> u8 {
    let mut probe = [0_u8; PROBE_BYTES];
    probe[0] = 1;
    probe[PROBE_BYTES - 1] = 1;
    std::hint::black_box(&mut probe);
    probe[0] + probe[PROBE_BYTES - 1]
}
