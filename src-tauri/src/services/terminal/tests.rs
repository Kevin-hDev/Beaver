#[cfg(test)]
mod tests {
    use crate::app_exit::AppExitCoordinator;
    use crate::services::terminal::pty_session::PtySession;
    use crate::services::terminal::PtyManager;
    use std::io::Read;
    use std::time::{Duration, Instant};
    use sysinfo::{Pid, System};

    fn process_is_running(pid: u32) -> bool {
        let mut system = System::new();
        system.refresh_processes(
            sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
            true,
        );
        system.process(Pid::from_u32(pid)).is_some()
    }

    #[test]
    fn test_pty_spawn() {
        let (session, _reader) = PtySession::spawn(None, 80, 24).expect("spawn failed");
        drop(session);
    }

    #[test]
    fn test_pty_spawn_with_cwd() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        let (session, _reader) = PtySession::spawn(Some(path), 80, 24).expect("spawn with cwd");
        drop(session);
    }

    #[test]
    fn test_pty_write() {
        let (session, _reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.write(b"echo hello\n").expect("write failed");
        drop(session);
    }

    #[test]
    fn test_pty_resize() {
        let (session, _reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.resize(40, 10).expect("resize failed");
        drop(session);
    }

    #[test]
    fn test_pty_kill() {
        let (mut session, _reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.kill().expect("kill failed");
    }

    #[test]
    #[cfg_attr(
        windows,
        ignore = "ClosePseudoConsole can block indefinitely during output teardown on Windows CI"
    )]
    fn test_pty_read_output() {
        let (session, mut reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.write(b"echo pty_test_marker\n").expect("write");

        let mut output = String::new();
        let mut buf = [0u8; 1024];
        let deadline = std::time::Instant::now() + Duration::from_secs(3);

        while std::time::Instant::now() < deadline {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if output.contains("pty_test_marker") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        assert!(
            output.contains("pty_test_marker"),
            "expected marker in output, got: {}",
            output
        );
        drop(session);
    }

    #[test]
    #[cfg_attr(
        windows,
        ignore = "ClosePseudoConsole can block indefinitely during multi-session teardown on Windows CI"
    )]
    fn test_multiple_independent_sessions() {
        let (_s1, _r1) = PtySession::spawn(None, 80, 24).expect("spawn 1");
        let (_s2, _r2) = PtySession::spawn(None, 80, 24).expect("spawn 2");
        let (_s3, _r3) = PtySession::spawn(None, 80, 24).expect("spawn 3");
    }

    #[tokio::test]
    async fn shutdown_reaps_a_real_shell_and_waits_for_terminal_threads() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = PtyManager::new(coordinator.work_supervisor());
        let (id, _) = manager
            .spawn_for_test(None, 80, 24)
            .expect("real PTY shell");
        let pid = manager.process_id_for_test(id).expect("shell process id");
        assert!(process_is_running(pid));

        assert!(
            manager
                .stop_and_wait(Instant::now() + Duration::from_secs(5))
                .await
        );

        assert!(!process_is_running(pid));
        assert_eq!(manager.active_sessions_for_test(), 0);
    }

    #[tokio::test]
    async fn shutdown_permanently_refuses_new_terminal_sessions() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = PtyManager::new(coordinator.work_supervisor());

        assert!(
            manager
                .stop_and_wait(Instant::now() + Duration::from_secs(1))
                .await
        );
        assert_eq!(
            manager.spawn_for_test(None, 80, 24).unwrap_err(),
            "terminal-shutting-down"
        );
    }

    #[test]
    fn terminal_capacity_remains_sixteen_sessions() {
        assert_eq!(PtyManager::MAX_PTY_SESSIONS, 16);
    }
}
