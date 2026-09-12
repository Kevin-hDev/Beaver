use super::stream_recovery_record::*;
use super::stream_recovery_store::*;
use super::stream_recovery_store_discovery::list_session_ids;

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

#[tokio::test]
async fn fifth_log_for_one_session_is_refused() {
    let session_id = uuid::Uuid::new_v4().to_string();
    let mut paths = Vec::new();
    for _ in 0..MAX_LOGS_PER_SESSION {
        let mut value = header();
        value.session_id.clone_from(&session_id);
        let (path, file) = create(&value).await.unwrap();
        drop(file);
        paths.push(path);
    }
    let mut extra = header();
    extra.session_id.clone_from(&session_id);

    assert!(create(&extra).await.is_err());
    for path in paths {
        remove(path).await.unwrap();
    }
}

#[tokio::test]
async fn oversized_log_is_refused_before_record_parsing() {
    let value = header();
    let (path, file) = create(&value).await.unwrap();
    file.set_len(MAX_LOG_BYTES + 1).unwrap();
    drop(file);

    assert!(visit_records(&path, |_| Ok(())).is_err());
    remove(path).await.unwrap();
}

#[test]
fn discovery_refuses_more_than_its_entry_budget() {
    let root = tempfile::tempdir().unwrap();
    for index in 0..=MAX_DISCOVERY_ENTRIES {
        std::fs::create_dir(root.path().join(index.to_string())).unwrap();
    }

    assert!(list_session_ids(root.path()).is_err());
}
