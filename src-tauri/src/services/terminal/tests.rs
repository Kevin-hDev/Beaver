#[cfg(test)]
mod tests {
    use crate::services::terminal::pty_session::PtySession;
    use std::io::Read;
    use std::sync::mpsc;
    use std::time::Duration;

    #[cfg(windows)]
    const MARKER_COMMAND: &[u8] = b"Write-Output pty_test_marker; exit\r\n";
    #[cfg(not(windows))]
    const MARKER_COMMAND: &[u8] = b"printf 'pty_test_marker\\n'; exit\n";

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
    fn test_pty_read_output() {
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
        let output = receiver.recv_timeout(Duration::from_secs(5));
        let _ = session.kill();
        if output.is_ok() {
            reader_thread.join().expect("reader thread");
        }
        let output = output.expect("PTY output timed out");

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

        let results = [s1.kill(), s2.kill(), s3.kill()];
        drop((r1, r2, r3));
        assert!(results.iter().all(Result::is_ok));
    }
}
