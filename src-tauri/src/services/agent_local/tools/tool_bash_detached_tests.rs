#[cfg(unix)]
#[tokio::test]
async fn detached_pipe_holder_preserves_the_parent_exit_status() {
    let directory = tempfile::tempdir().expect("tempdir");
    let started = std::time::Instant::now();

    let output = super::execute_shell("(sleep 8 &) &", directory.path(), Some(12))
        .await
        .expect("shell output");

    assert_eq!(output.exit_code, 0);
    assert!(output.output_incomplete);
    assert!(started.elapsed() < std::time::Duration::from_secs(7));
}
