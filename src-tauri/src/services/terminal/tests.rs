#[cfg(test)]
mod tests {
    use crate::services::terminal::pty_session::PtySession;
    use std::io::Read;
    use std::time::Duration;

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
    fn test_multiple_independent_sessions() {
        let (_s1, _r1) = PtySession::spawn(None, 80, 24).expect("spawn 1");
        let (_s2, _r2) = PtySession::spawn(None, 80, 24).expect("spawn 2");
        let (_s3, _r3) = PtySession::spawn(None, 80, 24).expect("spawn 3");
    }
}
