#[cfg(test)]
mod tests {
    use crate::services::terminal::pty_session::PtySession;
    use std::io::Read;
    use std::sync::{mpsc, Mutex, MutexGuard};
    use std::time::Duration;

    static PTY_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[cfg(windows)]
    const MARKER_COMMAND: &[u8] = b"Write-Output pty_test_marker; exit\r\n";
    #[cfg(not(windows))]
    const MARKER_COMMAND: &[u8] = b"printf 'pty_test_marker\\n'; exit\n";

    fn pty_test_guard() -> MutexGuard<'static, ()> {
        PTY_TEST_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn kill_session(mut session: PtySession, reader: Box<dyn Read + Send>) {
        drop(reader);
        session.kill().expect("kill failed");
    }

    #[test]
    fn test_pty_spawn() {
        let _guard = pty_test_guard();
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn failed");
        kill_session(session, reader);
    }

    #[test]
    fn test_pty_spawn_with_cwd() {
        let _guard = pty_test_guard();
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().to_str().unwrap();
        let (session, reader) = PtySession::spawn(Some(path), 80, 24).expect("spawn with cwd");
        kill_session(session, reader);
    }

    #[test]
    fn test_pty_write() {
        let _guard = pty_test_guard();
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.write(b"echo hello\n").expect("write failed");
        kill_session(session, reader);
    }

    #[test]
    fn test_pty_resize() {
        let _guard = pty_test_guard();
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        session.resize(40, 10).expect("resize failed");
        kill_session(session, reader);
    }

    #[test]
    fn test_pty_kill() {
        let _guard = pty_test_guard();
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        kill_session(session, reader);
    }

    #[test]
    fn test_pty_read_output() {
        let _guard = pty_test_guard();
        let (mut session, mut reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        let (sender, receiver) = mpsc::sync_channel(1);
        let reader_thread = std::thread::spawn(move || {
            let mut output = String::new();
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        output.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if output.contains("pty_test_marker") {
                            break;
                        }
                    }
                }
            }
            let _ = sender.send(output);
        });

        session.write(MARKER_COMMAND).expect("write");
        let output = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("PTY output timed out");
        reader_thread.join().expect("reader thread");
        session.kill().expect("kill failed");

        assert!(
            output.contains("pty_test_marker"),
            "expected marker in output, got: {}",
            output
        );
    }

    #[test]
    fn test_multiple_independent_sessions() {
        let _guard = pty_test_guard();
        let (s1, r1) = PtySession::spawn(None, 80, 24).expect("spawn 1");
        let (s2, r2) = PtySession::spawn(None, 80, 24).expect("spawn 2");
        let (s3, r3) = PtySession::spawn(None, 80, 24).expect("spawn 3");

        kill_session(s1, r1);
        kill_session(s2, r2);
        kill_session(s3, r3);
    }
}
