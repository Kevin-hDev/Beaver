#[cfg(test)]
mod tests {
    use crate::services::terminal::pty_session::PtySession;
    use std::io::Read;
    use std::sync::mpsc;
    use std::thread::JoinHandle;
    use std::time::Duration;

    #[cfg(windows)]
    const MARKER_COMMAND: &[u8] = b"Write-Output pty_test_marker; exit\r\n";
    #[cfg(not(windows))]
    const MARKER_COMMAND: &[u8] = b"printf 'pty_test_marker\\n'; exit\n";
    const MAX_TEST_OUTPUT_BYTES: usize = 65_536;

    fn drain_reader(mut reader: Box<dyn Read + Send>) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let mut buf = [0u8; 1024];
            while matches!(reader.read(&mut buf), Ok(1..)) {}
        })
    }

    fn close_session(mut session: PtySession, reader: Box<dyn Read + Send>) {
        let reader_thread = drain_reader(reader);
        session.kill().expect("kill failed");
        drop(session);
        reader_thread.join().expect("reader thread");
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
        let (session, reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        close_session(session, reader);
    }

    #[test]
    fn test_pty_read_output() {
        let (mut session, mut reader) = PtySession::spawn(None, 80, 24).expect("spawn");
        let (sender, receiver) = mpsc::sync_channel(1);
        let reader_thread = std::thread::spawn(move || {
            let mut output = String::new();
            let mut buf = [0u8; 1024];
            let mut marker_sent = false;
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if output.len() < MAX_TEST_OUTPUT_BYTES {
                            let remaining = MAX_TEST_OUTPUT_BYTES - output.len();
                            output.push_str(&String::from_utf8_lossy(&buf[..n.min(remaining)]));
                        }
                        if !marker_sent && output.contains("pty_test_marker") {
                            let _ = sender.send(output.clone());
                            marker_sent = true;
                        }
                    }
                }
            }
            if !marker_sent {
                let _ = sender.send(output);
            }
        });

        session.write(MARKER_COMMAND).expect("write");
        let output = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("PTY output timed out");
        session.kill().expect("kill failed");
        drop(session);
        reader_thread.join().expect("reader thread");

        assert!(
            output.contains("pty_test_marker"),
            "expected marker in output, got: {}",
            output
        );
    }

    #[test]
    fn test_multiple_independent_sessions() {
        let (mut s1, r1) = PtySession::spawn(None, 80, 24).expect("spawn 1");
        let (mut s2, r2) = PtySession::spawn(None, 80, 24).expect("spawn 2");
        let (mut s3, r3) = PtySession::spawn(None, 80, 24).expect("spawn 3");
        let readers = [drain_reader(r1), drain_reader(r2), drain_reader(r3)];

        s1.kill().expect("kill 1");
        s2.kill().expect("kill 2");
        s3.kill().expect("kill 3");
        drop((s1, s2, s3));
        for reader in readers {
            reader.join().expect("reader thread");
        }
    }
}
