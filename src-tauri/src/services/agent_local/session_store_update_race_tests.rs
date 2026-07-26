use super::{session_store, session_store_updates};

#[tokio::test]
async fn working_dir_update_cannot_overwrite_a_concurrent_correction() {
    let session = session_store::create_full("Update race", "llama3", "ollama", false, None)
        .await
        .expect("create session");
    let target = crate::services::paths::data_dir();
    let expected = target
        .canonicalize()
        .expect("canonical target")
        .to_string_lossy()
        .to_string();
    let (loaded_tx, loaded_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let update_id = session.id.clone();
    let update_target = target.to_string_lossy().to_string();
    let update = tokio::spawn(async move {
        session_store_updates::update_working_dir_with_after_load(
            &update_id,
            &update_target,
            move || async move {
                let _ = loaded_tx.send(());
                let _ = release_rx.await;
            },
        )
        .await
    });
    loaded_rx.await.expect("update loaded session");
    let correction_id = session.id.clone();
    let correction = tokio::spawn(async move {
        super::session_store_messages::add_redeployment_prompt(
            &correction_id,
            "correction concurrente",
        )
        .await
    });
    let mut correction = Box::pin(correction);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), &mut correction)
        .await
        .is_err());

    let _ = release_tx.send(());
    update
        .await
        .expect("join update")
        .expect("update working dir");
    correction
        .await
        .expect("join correction")
        .expect("persist correction");
    let saved = session_store::get(&session.id).await.expect("load session");
    assert_eq!(saved.working_dir, expected);
    assert_eq!(saved.subagent_queued_prompts, vec!["correction concurrente"]);
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}
