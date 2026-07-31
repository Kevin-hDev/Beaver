use super::*;

#[cfg(unix)]
#[tokio::test]
async fn cancellation_interrupts_the_direct_process() {
    let cancel = CancellationToken::new();
    let command_cancel = cancel.clone();
    let task = tokio::spawn(async move {
        run(
            &["/bin/sh".into(), "-c".into(), "sleep 30".into()],
            Path::new("/tmp"),
            None,
            command_cancel,
        )
        .await
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    cancel.cancel();
    let output = task.await.expect("task").expect("output");

    assert_ne!(output.exit_code, 0);
    assert!(!output.timed_out);
}

#[cfg(unix)]
#[tokio::test]
async fn direct_output_is_bounded() {
    let output = run(
        &[
            "/bin/sh".into(),
            "-c".into(),
            "yes x | head -c 100000".into(),
        ],
        Path::new("/tmp"),
        None,
        CancellationToken::new(),
    )
    .await
    .expect("output");

    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.len() < MAX_OUTPUT_BYTES);
    assert!(output.stdout.contains("sortie tronquée"));
}
