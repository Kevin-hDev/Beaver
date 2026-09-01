use super::caller::TerminalOwner;
use super::manager::{PtyManager, NEXT_ID};
use super::session_handle::{SessionControl, SessionHandle, SessionOps};
use super::PtyChannelEvent;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

impl PtyManager {
    pub(in crate::services::terminal) fn spawn_for_test(
        &self,
        owner: &TerminalOwner,
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
    ) -> Result<(u32, String), String> {
        self.spawn_with_sink(owner, cwd, cols, rows, |_| Ok(()))
    }

    pub(crate) fn spawn_with_test_sink(
        &self,
        owner: &TerminalOwner,
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
        sink: impl Fn(PtyChannelEvent) -> Result<(), ()> + Send + 'static,
    ) -> Result<(u32, String), String> {
        self.spawn_with_sink(owner, cwd, cols, rows, sink)
    }

    pub(in crate::services::terminal) fn insert_session_for_test(
        &self,
        owner: &TerminalOwner,
        operations: Box<dyn SessionOps>,
        control: SessionControl,
        token: &str,
    ) -> (u32, String) {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let token = zeroize::Zeroizing::new(token.to_string());
        let token_copy = token.to_string();
        self.state.lock().unwrap().sessions.insert(
            id,
            Arc::new(SessionHandle::new(
                owner.clone(),
                operations,
                control,
                token,
            )),
        );
        (id, token_copy)
    }

    pub(in crate::services::terminal) fn manager_lock_is_available_for_test(&self) -> bool {
        self.state.try_lock().is_ok()
    }

    pub(in crate::services::terminal) fn process_id_for_test(&self, id: u32) -> Option<u32> {
        self.state.lock().ok()?.sessions.get(&id)?.process_id()
    }

    pub(in crate::services::terminal) fn active_sessions_for_test(&self) -> usize {
        self.state
            .lock()
            .map(|state| state.sessions.len())
            .unwrap_or(Self::MAX_PTY_SESSIONS)
    }
}

#[cfg(test)]
mod tests {
    use crate::app_exit::AppExitCoordinator;
    use crate::services::terminal::caller::{authorize, TerminalOwner};
    use crate::services::terminal::cwd_resolver::resolve_with;
    use crate::services::terminal::limits::MAX_IN_FLIGHT_FRAMES;
    use crate::services::terminal::pty_session::PtySession;
    use crate::services::terminal::shutdown;
    use crate::services::terminal::PtyManager;
    use std::collections::VecDeque;
    use std::io::Read;
    use std::path::Path;
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

    #[cfg(unix)]
    fn continuous_output_command() -> &'static [u8] {
        b"yes x\n"
    }

    #[cfg(windows)]
    fn continuous_output_command() -> &'static [u8] {
        b"while ($true) { Write-Output 'x' }\r\n"
    }

    #[cfg(unix)]
    fn final_output_command() -> &'static [u8] {
        b"i=0; while [ \"$i\" -lt 10000 ]; do printf 'line\\n'; i=$((i+1)); done; printf 'BEAVER_FINAL_\\360\\237\\246\\253\\n'; exit\n"
    }

    #[cfg(windows)]
    fn final_output_command() -> &'static [u8] {
        b"1..10000 | ForEach-Object { Write-Output 'line' }; Write-Output ('BEAVER_FINAL_' + [char]::ConvertFromUtf32(0x1F9AB)); exit\r\n"
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
        let (session, reader) =
            PtySession::spawn(Some(tmp.path()), 80, 24).expect("spawn with cwd");
        close_session(session, reader);
    }

    #[test]
    fn test_pty_rejects_a_relative_cwd_path() {
        let error = PtySession::spawn(Some(Path::new("relative")), 80, 24)
            .err()
            .expect("relative cwd must fail");

        assert_eq!(error, "terminal-cwd-invalid");
    }

    #[tokio::test]
    async fn resolved_project_key_is_passed_to_the_manager_as_a_path() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("Projet espace é");
        std::fs::create_dir(&project).unwrap();
        let project_string = project.to_string_lossy().into_owned();
        let resolved = resolve_with("project-a", Path::new("/"), |_| async move {
            Ok(Some(project_string))
        })
        .await
        .unwrap();
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = PtyManager::new(coordinator.work_supervisor());
        let owner = authorize("main").expect("main owner");

        manager
            .spawn_for_test(&owner, Some(resolved.as_path()), 80, 24)
            .expect("spawn in resolved project");

        assert!(
            manager
                .stop_and_wait(Instant::now() + Duration::from_secs(5))
                .await
        );
    }

    #[tokio::test]
    async fn default_group_uses_the_canonical_home_path() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("child")).unwrap();
        let home_with_parent = root.path().join("child/..");

        let resolved = resolve_with("__default__", &home_with_parent, unreachable_find)
            .await
            .unwrap();

        assert_eq!(resolved, dunce::canonicalize(root.path()).unwrap());
    }

    #[tokio::test]
    async fn invalid_group_key_returns_the_public_cwd_error() {
        assert_eq!(
            resolve_with("bad\nkey", Path::new("/"), unreachable_find).await,
            Err("terminal-cwd-invalid".to_string())
        );
    }

    async fn unreachable_find(_: String) -> Result<Option<String>, String> {
        panic!("default and invalid groups must not query the project registry")
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
        let (chunks, received) = std::sync::mpsc::sync_channel(4);
        let reader_thread = std::thread::spawn(move || {
            let mut buffer = [0_u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => {
                        if chunks.send(buffer[..read].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        let mut output = String::new();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !output.contains("pty_test_marker") {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match received.recv_timeout(remaining) {
                Ok(chunk) => output.push_str(&String::from_utf8_lossy(&chunk)),
                Err(error) => {
                    drop(received);
                    drop(session);
                    reader_thread.join().expect("reader worker");
                    panic!("PTY output deadline reached: {error}");
                }
            }
        }
        drop(received);
        drop(session);
        reader_thread.join().expect("reader worker");
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
        let owner = authorize("main").expect("main owner");
        let (id, _) = manager
            .spawn_for_test(&owner, None, 80, 24)
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

    #[cfg(target_os = "macos")]
    #[test]
    fn terminal_shell_dies_with_parent_macos() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = PtyManager::new(coordinator.work_supervisor());
        let owner = authorize("main").expect("main owner");
        let (id, token) = manager
            .spawn_for_test(&owner, None, 80, 24)
            .expect("real PTY shell");
        let pid = manager.process_id_for_test(id).expect("shell process id");
        assert!(process_is_running(pid));

        manager.kill(&owner, id, &token).expect("close PTY master");

        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline && process_is_running(pid) {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!process_is_running(pid));
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
        let owner = authorize("main").expect("main owner");

        assert!(
            manager
                .stop_and_wait(Instant::now() + Duration::from_secs(1))
                .await
        );
        assert_eq!(
            manager.spawn_for_test(&owner, None, 80, 24).unwrap_err(),
            "terminal-shutting-down"
        );
    }

    #[test]
    fn terminal_capacity_remains_sixteen_sessions() {
        assert_eq!(PtyManager::MAX_PTY_SESSIONS, 16);
    }

    #[test]
    fn terminal_owner_rejects_every_operation_from_another_window() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = PtyManager::new(coordinator.work_supervisor());
        let main = authorize("main").expect("main owner");
        let other = TerminalOwner::for_test("secondary").expect("secondary owner");
        let (id, token) = manager
            .spawn_for_test(&main, None, 80, 24)
            .expect("terminal owned by main");

        assert_eq!(
            manager.write(&other, id, &token, b"forbidden"),
            Err("terminal-not-authorized".to_string())
        );
        assert_eq!(manager.active_sessions_for_test(), 1);
        assert_eq!(
            manager.resize(&other, id, &token, 40, 10),
            Err("terminal-not-authorized".to_string())
        );
        assert_eq!(manager.active_sessions_for_test(), 1);
        assert_eq!(
            manager.acknowledge(&other, id, &token, 1),
            Err("terminal-not-authorized".to_string())
        );
        assert_eq!(manager.active_sessions_for_test(), 1);
        assert_eq!(
            manager.kill(&other, id, &token),
            Err("terminal-not-authorized".to_string())
        );
        assert_eq!(manager.active_sessions_for_test(), 1);

        assert_eq!(manager.kill(&main, id, &token), Ok(()));
        assert_eq!(manager.active_sessions_for_test(), 0);
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

    #[test]
    fn terminal_close_stays_bounded_when_output_window_is_full() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = PtyManager::new(coordinator.work_supervisor());
        let owner = authorize("main").expect("main owner");
        let (saturated_tx, saturated) = std::sync::mpsc::sync_channel(1);
        let (id, token) = manager
            .spawn_with_test_sink(&owner, None, 80, 24, move |event| {
                if event.sequence == Some(MAX_IN_FLIGHT_FRAMES as u32) {
                    let _ = saturated_tx.try_send(());
                }
                Ok(())
            })
            .expect("terminal with test sink");
        let pid = manager
            .process_id_for_test(id)
            .expect("terminal process id");
        manager
            .write(&owner, id, &token, continuous_output_command())
            .expect("start continuous output");
        saturated
            .recv_timeout(Duration::from_secs(10))
            .expect("output window reaches 256 unacknowledged frames");

        let (closed_tx, closed) = std::sync::mpsc::sync_channel(1);
        let closer = {
            let manager = manager.clone();
            let owner = owner.clone();
            let token = token.clone();
            std::thread::spawn(move || {
                closed_tx
                    .send(manager.kill(&owner, id, &token))
                    .expect("report terminal close");
            })
        };

        assert_eq!(closed.recv_timeout(Duration::from_secs(3)), Ok(Ok(())));
        closer.join().expect("terminal close worker");
        assert!(!process_is_running(pid));
        assert_eq!(manager.active_sessions_for_test(), 0);
    }

    #[test]
    fn final_output_precedes_exit_event() {
        const MARKER: &str = "BEAVER_FINAL_🦫";
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let manager = PtyManager::new(coordinator.work_supervisor());
        let owner = authorize("main").expect("main owner");
        let (events_tx, events) = std::sync::mpsc::sync_channel(256);
        let (id, token) = manager
            .spawn_with_test_sink(&owner, None, 80, 24, move |event| {
                events_tx.send(event).map_err(|_| ())
            })
            .expect("terminal with output channel");
        let pid = manager
            .process_id_for_test(id)
            .expect("terminal process id");
        manager
            .write(&owner, id, &token, final_output_command())
            .expect("start finite output");

        let marker_length = MARKER.chars().count();
        let mut suffix = VecDeque::with_capacity(marker_length);
        let mut marker_seen = false;
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let event = events
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("terminal emits final output and exit");
            if event.is_exit {
                assert_eq!(event.sequence, None);
                assert!(marker_seen, "Unicode marker must precede exit");
                break;
            }
            for character in event.data.chars() {
                if suffix.len() == marker_length {
                    suffix.pop_front();
                }
                suffix.push_back(character);
                marker_seen |= suffix.iter().copied().eq(MARKER.chars());
            }
            manager
                .acknowledge(&owner, id, &token, event.sequence.expect("data sequence"))
                .expect("acknowledge terminal output");
        }

        manager
            .kill(&owner, id, &token)
            .expect("close completed terminal");
        assert!(matches!(
            events.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty | std::sync::mpsc::TryRecvError::Disconnected)
        ));
        assert!(!process_is_running(pid));
        assert_eq!(manager.active_sessions_for_test(), 0);
    }
}
