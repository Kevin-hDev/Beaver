#[cfg(test)]
mod tests {
    use crate::app_exit::AppExitCoordinator;
    use crate::services::terminal::pty_session::PtySession;
    use crate::services::terminal::shutdown;
    use crate::services::terminal::PtyManager;
    use std::io::Read;
    use std::sync::{Arc, Condvar, Mutex};
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

    /// Ferme une session dans l'ordre que l'application produit : le lecteur,
    /// puis la session.
    ///
    /// Le lecteur partage le descripteur maître avec la session. Tant qu'un
    /// détenteur le garde ouvert sans le drainer, le noyau retient le shell
    /// dans sa sortie, et la fermeture ne rend la main qu'en abandonnant au
    /// bout de son délai — le test passe alors sans avoir exercé la fermeture.
    /// L'application n'a pas ce cas : son fil beaver-pty-reader lit en continu
    /// jusqu'à la fermeture du maître.
    fn close_session(session: PtySession, reader: Box<dyn Read + Send>) {
        drop(reader);
        drop(session);
    }

    #[test]
    fn test_pty_spawn() {
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn failed");
        close_session(session, reader);
    }

    #[test]
    fn test_pty_spawn_with_cwd() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        let (session, reader) = PtySession::spawn(Some(path), 80, 24).expect("spawn with cwd");
        close_session(session, reader);
    }

    #[test]
    fn test_pty_write() {
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.write(b"echo hello\n").expect("write failed");
        close_session(session, reader);
    }

    #[test]
    fn test_pty_resize() {
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.resize(40, 10).expect("resize failed");
        close_session(session, reader);
    }

    #[test]
    fn test_pty_kill() {
        let (mut session, reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        drop(reader);
        session.kill().expect("kill failed");
    }

    #[test]
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
        close_session(session, reader);
    }

    #[test]
    fn test_multiple_independent_sessions() {
        let (s1, r1) = PtySession::spawn(None, 80, 24).expect("spawn 1");
        let (s2, r2) = PtySession::spawn(None, 80, 24).expect("spawn 2");
        let (s3, r3) = PtySession::spawn(None, 80, 24).expect("spawn 3");
        close_session(s1, r1);
        close_session(s2, r2);
        close_session(s3, r3);
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

    #[test]
    #[cfg(windows)]
    fn windows_terminal_process_is_confined_by_beaver() {
        let (session, _reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        let pid = session.process_id().expect("shell process id");

        assert!(crate::services::owned_process::OwnedProcess::is_confined_for_test(pid));

        drop(session);
    }

    #[test]
    #[cfg(windows)]
    fn windows_full_output_pipe_does_not_block_pty_close() {
        let (finished, result) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let (session, _unread_output) =
                PtySession::spawn(None, 80, 24).expect("spawn flooding shell");
            let pid = session.process_id().expect("shell process id");
            session
                .write(b"1..10000 | % { 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx' }\r\n")
                .expect("start output flood");
            std::thread::sleep(Duration::from_millis(250));
            drop(session);
            finished.send(pid).expect("report bounded close");
        });

        let pid = result
            .recv_timeout(Duration::from_secs(3))
            .expect("closing a full ConPTY pipe must stay bounded");
        assert!(!process_is_running(pid));
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

    #[test]
    fn timed_out_terminal_close_does_not_pin_tokio_runtime_shutdown() {
        let release = Arc::new((Mutex::new(false), Condvar::new()));
        let operation_release = Arc::clone(&release);
        let (dropped, observed) = std::sync::mpsc::sync_channel(1);
        let runtime_owner = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .expect("test runtime");
            assert!(!runtime.block_on(shutdown::run_until(
                Instant::now() + Duration::from_millis(20),
                move || {
                    let (lock, wake) = &*operation_release;
                    let mut released = lock.lock().expect("release lock");
                    while !*released {
                        released = wake.wait(released).expect("release wait");
                    }
                },
            )));
            drop(runtime);
            dropped.send(()).expect("report runtime drop");
        });

        let runtime_dropped = observed.recv_timeout(Duration::from_millis(250));
        let (lock, wake) = &*release;
        *lock.lock().expect("release lock") = true;
        wake.notify_one();
        runtime_owner.join().expect("runtime owner");

        assert!(runtime_dropped.is_ok(), "Tokio waited for timed-out close");
    }
}
