use super::{isolated_toolset_in, purge_stale_runtime_in};
use serde_json::json;

#[tokio::test]
async fn fixture_tools_only_read_and_write_their_private_note() {
    let runtime = tempfile::tempdir().expect("runtime");
    let mut tools = isolated_toolset_in(runtime.path().to_path_buf())
        .await
        .expect("toolset");

    assert_eq!(
        tools
            .execute("fixture.write_note", &json!({ "value": "fixture" }))
            .await
            .expect("write"),
        json!({ "written": true })
    );
    assert_eq!(
        tools
            .execute("fixture.read_note", &json!({}))
            .await
            .expect("read"),
        json!({ "value": "fixture" })
    );
}

#[tokio::test]
async fn unknown_tools_and_paths_are_rejected_before_writing() {
    let runtime = tempfile::tempdir().expect("runtime");
    let outside = runtime.path().join("outside");
    let mut tools = isolated_toolset_in(runtime.path().to_path_buf())
        .await
        .expect("toolset");

    assert!(tools.execute("shell", &json!({})).await.is_err());
    assert!(tools
        .execute(
            "fixture.write_note",
            &json!({ "value": "ok", "path": "../outside" })
        )
        .await
        .is_err());
    assert!(!outside.exists());
}

#[tokio::test]
async fn fixture_directory_is_removed_when_the_toolset_is_dropped_after_a_failure() {
    let runtime = tempfile::tempdir().expect("runtime");
    let path = {
        let mut tools = isolated_toolset_in(runtime.path().to_path_buf())
            .await
            .expect("toolset");
        tools
            .execute("fixture.write_note", &json!({ "value": "fixture" }))
            .await
            .expect("write");
        assert!(tools
            .execute("fixture.read_note", &json!({ "unexpected": true }))
            .await
            .is_err());
        tools.root_for_test()
    };

    assert!(!path.exists());
}

#[test]
fn startup_cleanup_removes_only_bounded_fixture_directories() {
    let runtime = tempfile::tempdir().expect("runtime");
    let stale = runtime.path().join("fixture-stale");
    std::fs::create_dir(&stale).expect("stale fixture");
    std::fs::write(stale.join("fixture-note.txt"), b"stale").expect("stale note");

    purge_stale_runtime_in(runtime.path()).expect("purge stale fixture");

    assert!(!stale.exists());
}

#[test]
fn startup_cleanup_fails_closed_on_unrelated_entries() {
    let runtime = tempfile::tempdir().expect("runtime");
    let unrelated = runtime.path().join("keep-me");
    std::fs::create_dir(&unrelated).expect("unrelated directory");

    assert!(purge_stale_runtime_in(runtime.path()).is_err());
    assert!(unrelated.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn fixture_tools_do_not_follow_a_symlink_inside_the_private_directory() {
    use std::os::unix::fs::symlink;

    let runtime = tempfile::tempdir().expect("runtime");
    let outside = runtime.path().join("outside");
    std::fs::write(&outside, "safe").expect("outside marker");
    let mut tools = isolated_toolset_in(runtime.path().to_path_buf())
        .await
        .expect("toolset");
    std::fs::remove_file(tools.note_path_for_test()).ok();
    symlink(&outside, tools.note_path_for_test()).expect("link");

    assert!(tools
        .execute("fixture.read_note", &json!({}))
        .await
        .is_err());
    assert_eq!(
        std::fs::read_to_string(&outside).expect("outside body"),
        "safe"
    );
}
