use super::*;

fn entry(index: usize) -> AuditEntry {
    AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        channel: "telegram".into(),
        account_id: "bot1".into(),
        user_hash: format!("{index:016x}"),
        action: AuditAction::MessageReceived,
        security_decision: None,
        error: None,
    }
}

#[test]
fn hmac_is_stable_per_key_and_changes_with_key() {
    let a = hash_user_id_with_key("user42", &[1_u8; 32]).unwrap();
    let b = hash_user_id_with_key("user42", &[1_u8; 32]).unwrap();
    let c = hash_user_id_with_key("user42", &[2_u8; 32]).unwrap();
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.len(), 16);
}

#[test]
fn sanitize_error_never_keeps_internal_details() {
    let sanitized = sanitize_error("Bearer secret at /private/path");
    assert_eq!(sanitized, "operation_failed");
}

#[test]
fn concurrent_audit_writes_are_complete_json_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let mut workers = Vec::new();
    for worker in 0..8 {
        let path = path.clone();
        workers.push(std::thread::spawn(move || {
            for offset in 0..25 {
                let line = serde_json::to_string(&entry(worker * 25 + offset)).unwrap();
                append_serialized(&path, &line, 30).unwrap();
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let content = std::fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 200);
    assert!(lines
        .iter()
        .all(|line| serde_json::from_str::<AuditEntry>(line).is_ok()));
}

#[test]
fn audit_write_failure_is_returned_to_the_caller() {
    let dir = tempfile::tempdir().expect("tempdir");

    let result = log_audit_to_path(&entry(1), dir.path());

    assert!(result.is_err());
}
