use super::*;
use crate::services::agent_local::{session_store, session_store_updates};

#[tokio::test]
async fn project_cleanup_revalidates_the_selected_project_under_lock() {
    let mut session = session_store::create_full(
        "Project race",
        "llama3",
        "ollama",
        false,
        Some("deleted-project".to_string()),
    )
    .await
    .expect("create session");
    session.project_id = Some("deleted-project".to_string());
    session_store::save(&session).await.expect("seed project");
    let (selected_tx, selected_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let cleanup = tokio::spawn(async move {
        clear_project_id_with_after_list("deleted-project", move || async move {
            let _ = selected_tx.send(());
            let _ = release_rx.await;
        })
        .await
    });
    selected_rx.await.expect("cleanup selected stale metadata");

    session_store_updates::update_locked(&session.id, |current| {
        current.project_id = Some("replacement-project".to_string());
    })
    .await
    .expect("move session to replacement project");
    let _ = release_tx.send(());
    cleanup.await.expect("join cleanup").expect("cleanup");

    let saved = session_store::get(&session.id).await.expect("reload");
    assert_eq!(saved.project_id.as_deref(), Some("replacement-project"));
    session_store::delete_one(&session.id).await.expect("cleanup session");
}
