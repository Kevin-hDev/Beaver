use super::stream_recovery_record::*;
use super::stream_recovery_store::*;

fn header() -> StreamRecoveryHeader {
    StreamRecoveryHeader {
        version: STREAM_RECOVERY_VERSION,
        process_instance_id: uuid::Uuid::new_v4().to_string(),
        session_id: uuid::Uuid::new_v4().to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        user_message_id: uuid::Uuid::new_v4().to_string(),
        assistant_message_id: uuid::Uuid::new_v4().to_string(),
        subagent_owner: None,
        created_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn private_log_round_trips_and_tolerates_a_cut_final_line() {
    let header = header();
    let (path, mut file) = create(&header).await.unwrap();
    use std::io::Write;
    writeln!(file, "{}", serde_json::to_string(&StreamRecoveryRecord::TurnReady { sequence: 1 }).unwrap()).unwrap();
    write!(file, "{{\"kind\":").unwrap();
    drop(file);

    let mut count = 0;
    visit_records(&path, |_| { count += 1; Ok(()) }).unwrap();
    assert_eq!(count, 2);
    remove(path).await.unwrap();
}

#[test]
fn invalid_identifiers_never_form_a_path() {
    assert!(path_for("../session", &uuid::Uuid::new_v4().to_string()).is_err());
    assert!(path_for(&uuid::Uuid::new_v4().to_string(), "../request").is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn append_rejects_symlinks_and_hardlinks() {
    let header = header();
    let (path, file) = create(&header).await.unwrap();
    drop(file);
    let target = tempfile::NamedTempFile::new().unwrap();
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(target.path(), &path).unwrap();
    assert!(open_append(&path).is_err());
    std::fs::remove_file(&path).unwrap();
    std::fs::hard_link(target.path(), &path).unwrap();
    assert!(open_append(&path).is_err());
    std::fs::remove_file(&path).unwrap();
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}
