use super::*;

fn capture_path(path: &Path) -> Option<FileState> {
    let mut remaining = MAX_DIFF_FILE_BYTES as usize;
    capture(path, &mut remaining)
}

#[test]
fn records_full_file_replacement_as_old_and_new_lines() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("sample.md");
    std::fs::write(&path, "old line\nkept\n").expect("initial write");
    let before = capture_path(&path);

    std::fs::write(&path, "new line\nkept\n").expect("replacement");
    let after = capture_path(&path);
    let change = build_change(&path, before.as_ref(), after.as_ref()).expect("change");

    assert!(matches!(change.status, ToolFileChangeStatus::Modified));
    assert_eq!((change.additions, change.deletions), (1, 1));
    let lines = change
        .diff
        .expect("diff")
        .hunks
        .into_iter()
        .flat_map(|h| h.lines);
    assert!(lines
        .clone()
        .any(|line| line.kind == "deleted" && line.content == "old line"));
    assert!(lines
        .into_iter()
        .any(|line| line.kind == "added" && line.content == "new line"));
}

#[test]
fn records_deleted_file_with_its_previous_content() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("deleted.md");
    std::fs::write(&path, "first\nsecond\n").expect("initial write");
    let before = capture_path(&path);

    std::fs::remove_file(&path).expect("delete");
    let change = build_change(&path, before.as_ref(), None).expect("change");

    assert!(matches!(change.status, ToolFileChangeStatus::Deleted));
    assert_eq!((change.additions, change.deletions), (0, 2));
    assert!(change
        .diff
        .expect("diff")
        .hunks
        .iter()
        .flat_map(|h| &h.lines)
        .all(|line| line.kind == "deleted"));
}

#[test]
fn stored_change_sample_has_a_strict_serialized_size_limit() {
    let dir = tempfile::tempdir().expect("tempdir");
    let changes = (0..200)
        .map(|index| {
            let path = dir.path().join(format!("file-{index}.txt"));
            let content = format!("{}\n", "x".repeat(8_000));
            std::fs::write(&path, content).expect("file");
            let after = capture_path(&path);
            build_change(&path, None, after.as_ref()).expect("change")
        })
        .collect();

    let (sample, incomplete) = bounded_sample(changes);
    let serialized = serde_json::to_vec(&sample).expect("serialized sample");

    assert!(incomplete);
    assert!(sample.len() <= MAX_STORED_FILE_CHANGES);
    assert!(serialized.len() <= MAX_STORED_FILE_CHANGES_BYTES);
}
