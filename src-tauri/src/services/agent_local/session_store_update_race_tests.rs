use super::{session_store, session_store_updates};

#[tokio::test]
async fn working_dir_update_cannot_overwrite_a_concurrent_correction() {
    let session = session_store::create_full("Update race", "llama3", "ollama", false, None)
        .await
        .expect("create session");
    let target = crate::services::paths::data_dir();
    let expected = dunce::canonicalize(&target)
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

#[tokio::test]
async fn runtime_refresh_keeps_an_automatic_workspace_hidden() {
    let session = session_store::create_full("Managed", "llama3", "ollama", false, None)
        .await
        .expect("create session");
    let root = tempfile::tempdir().expect("tempdir");

    session_store_updates::set_managed_working_dir(
        &session.id,
        root.path().to_string_lossy().as_ref(),
    )
    .await
    .expect("set managed directory");
    session_store_updates::refresh_working_dir(
        &session.id,
        root.path().to_string_lossy().as_ref(),
    )
    .await
    .expect("refresh managed directory");

    let saved = session_store::get(&session.id).await.expect("load session");
    assert!(saved.working_dir_managed);

    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn selecting_a_directory_makes_it_visible_again() {
    let session = session_store::create_full("Visible", "llama3", "ollama", false, None)
        .await
        .expect("create session");
    let root = tempfile::tempdir().expect("tempdir");

    session_store_updates::set_managed_working_dir(
        &session.id,
        root.path().to_string_lossy().as_ref(),
    )
    .await
    .expect("set managed directory");
    session_store_updates::update_working_dir(
        &session.id,
        root.path().to_string_lossy().as_ref(),
    )
    .await
    .expect("select directory");

    let saved = session_store::get(&session.id).await.expect("load session");
    assert!(!saved.working_dir_managed);

    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn fast_mode_update_and_rename_keep_both_mutations() {
    let session = session_store::create_full("Before", "gpt-5.6", "openai", false, None)
        .await
        .expect("create session");
    let (loaded_tx, loaded_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let update_id = session.id.clone();
    let update = tokio::spawn(async move {
        session_store_updates::update_fast_mode_with_after_load(
            &update_id,
            true,
            move || async move {
                let _ = loaded_tx.send(());
                let _ = release_rx.await;
            },
        )
        .await
    });
    loaded_rx.await.expect("fast update loaded session");

    let rename_id = session.id.clone();
    let rename = tokio::spawn(async move { session_store::rename(&rename_id, "After").await });
    let mut rename = Box::pin(rename);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), &mut rename)
        .await
        .is_err());

    let _ = release_tx.send(());
    update.await.expect("join update").expect("update fast mode");
    rename.await.expect("join rename").expect("rename session");
    let saved = session_store::get(&session.id).await.expect("load session");
    assert!(saved.fast_mode_enabled);
    assert_eq!(saved.name, "After");
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

async fn paused_project_assignment(
    session_id: String,
    loaded_tx: tokio::sync::oneshot::Sender<()>,
    release_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<bool, String> {
    session_store_updates::assign_project_with_after_load(
        &session_id,
        "project-new",
        move || async move {
            let _ = loaded_tx.send(());
            let _ = release_rx.await;
        },
    )
    .await
}

#[tokio::test]
async fn project_assignment_and_rename_keep_both_mutations() {
    let session = session_store::create_full("Before", "gpt-5.6", "openai", false, None)
        .await
        .expect("create session");
    let (loaded_tx, loaded_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let update = tokio::spawn(paused_project_assignment(
        session.id.clone(),
        loaded_tx,
        release_rx,
    ));
    loaded_rx.await.expect("project assignment loaded session");
    let rename_id = session.id.clone();
    let rename = tokio::spawn(async move { session_store::rename(&rename_id, "After").await });
    let mut rename = Box::pin(rename);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), &mut rename)
        .await
        .is_err());

    let _ = release_tx.send(());
    assert!(update.await.expect("join assignment").expect("assignment"));
    rename.await.expect("join rename").expect("rename session");
    let saved = session_store::get(&session.id).await.expect("load session");
    assert_eq!(saved.project_id.as_deref(), Some("project-new"));
    assert_eq!(saved.name, "After");
    session_store::delete_one(&session.id).await.expect("cleanup");
}

#[tokio::test]
async fn project_assignment_and_model_update_keep_both_mutations() {
    let session = session_store::create_full("Model", "gpt-5.6", "openai", false, None)
        .await
        .expect("create session");
    let (loaded_tx, loaded_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let update = tokio::spawn(paused_project_assignment(
        session.id.clone(),
        loaded_tx,
        release_rx,
    ));
    loaded_rx.await.expect("project assignment loaded session");
    let model_id = session.id.clone();
    let model = tokio::spawn(async move {
        session_store_updates::update_model(&model_id, "llama3", "ollama", None, Some(false))
            .await
    });
    let mut model = Box::pin(model);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), &mut model)
        .await
        .is_err());

    let _ = release_tx.send(());
    assert!(update.await.expect("join assignment").expect("assignment"));
    model.await.expect("join model").expect("update model");
    let saved = session_store::get(&session.id).await.expect("load session");
    assert_eq!(saved.project_id.as_deref(), Some("project-new"));
    assert_eq!((saved.model.as_str(), saved.provider.as_str()), ("llama3", "ollama"));
    session_store::delete_one(&session.id).await.expect("cleanup");
}

#[tokio::test]
async fn project_assignment_and_reasoning_update_keep_both_mutations() {
    let session = session_store::create_full("Reasoning", "gpt-5.6", "openai", false, None)
        .await
        .expect("create session");
    let (loaded_tx, loaded_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let update = tokio::spawn(paused_project_assignment(
        session.id.clone(),
        loaded_tx,
        release_rx,
    ));
    loaded_rx.await.expect("project assignment loaded session");
    let reasoning_id = session.id.clone();
    let reasoning = tokio::spawn(async move {
        session_store_updates::update_reasoning(
            &reasoning_id,
            Some("high".to_string()),
            Some(true),
        )
        .await
    });
    let mut reasoning = Box::pin(reasoning);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), &mut reasoning)
        .await
        .is_err());

    let _ = release_tx.send(());
    assert!(update.await.expect("join assignment").expect("assignment"));
    reasoning
        .await
        .expect("join reasoning")
        .expect("update reasoning");
    let saved = session_store::get(&session.id).await.expect("load session");
    assert_eq!(saved.project_id.as_deref(), Some("project-new"));
    assert_eq!(saved.reasoning_mode.as_deref(), Some("high"));
    session_store::delete_one(&session.id).await.expect("cleanup");
}

#[test]
fn user_session_writers_are_routed_through_the_shared_lock_gate() {
    let updates = include_str!("session_store_updates.rs");
    let store = include_str!("session_store.rs");
    let commands = include_str!("../../commands/agent_sessions.rs");
    let clone_git = include_str!("clone_git.rs");
    let clone_git_link = include_str!("clone_git_link.rs");
    let clone_git_cleanup = include_str!("clone_git_cleanup.rs");
    let tabs_git = include_str!("session_tabs_git.rs");
    let session_ops = include_str!("session_ops.rs");
    let session_mutations = include_str!("session_mutations.rs");
    let delegate_child = include_str!("tool_delegate_child.rs");

    assert_eq!(updates.matches("update_locked(id, |session|").count(), 3);
    assert!(store.contains("session_store_updates::update_locked(id, |session|"));
    assert!(commands.contains("session_ops::apply_metadata_patch"));
    assert_eq!(clone_git.matches("lock_session(clone_session_id)").count(), 2);
    assert!(clone_git_link.contains("lock_session(clone_session_id)"));
    assert!(clone_git_cleanup.contains("lock_session(session_id)"));
    assert!(tabs_git.contains("lock_session(&tab.session_id)"));
    assert!(session_ops.contains("session_store_updates::update_locked(&meta.id"));
    assert!(session_mutations.contains("session_store_updates::update_locked(id"));
    assert!(session_mutations.contains("session_store::lock_session(id)"));
    assert!(delegate_child.contains("lock_session(&child.id)"));
}
