use super::{admit_background_if_idle, BackgroundAdmissionError};
use crate::services::agent_local::session_store;
use std::collections::HashMap;
use std::time::Duration;
use tauri::Manager;
use tokio::sync::Mutex;

#[tokio::test]
async fn idle_background_admission_finishes_and_persists_its_diagnostic() {
    let session = session_store::create_full("Idle automation", "test-model", "ollama", true, None)
        .await
        .unwrap();
    let app = tauri::test::mock_builder()
        .manage(crate::ActiveStreams(Mutex::new(HashMap::new())))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();

    let admitted = tokio::time::timeout(
        Duration::from_secs(2),
        admit_background_if_idle(app.handle(), &session.id),
    )
    .await
    .expect("background admission must not reacquire its own session lock")
    .unwrap();

    let stored = session_store::get(&session.id).await.unwrap();
    let diagnostic = stored.diagnostic_runs.last().unwrap();
    assert_eq!(diagnostic.request_id, admitted.request_id);
    assert_eq!(diagnostic.generation, admitted.generation);
    assert_eq!(diagnostic.status, "running");
    let streams = app.state::<crate::ActiveStreams>();
    assert_eq!(streams.0.lock().await.len(), 1);
    admitted.cancel.cancel();
    streams.0.lock().await.clear();
    session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn concurrent_background_admissions_keep_exactly_one_active_stream() {
    let session =
        session_store::create_full("Racing automation", "test-model", "ollama", true, None)
            .await
            .unwrap();
    let app = tauri::test::mock_builder()
        .manage(crate::ActiveStreams(Mutex::new(HashMap::new())))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();

    let (first, second) = tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(
            admit_background_if_idle(app.handle(), &session.id),
            admit_background_if_idle(app.handle(), &session.id),
        )
    })
    .await
    .expect("competing admissions must finish without a lock cycle");

    assert_ne!(first.is_ok(), second.is_ok());
    let rejected = if first.is_ok() { &second } else { &first };
    assert!(matches!(
        rejected,
        Err(BackgroundAdmissionError::Busy | BackgroundAdmissionError::Unavailable)
    ));
    let admitted = first.or(second).unwrap();
    let streams = app.state::<crate::ActiveStreams>();
    let active = streams.0.lock().await;
    assert_eq!(active.len(), 1);
    assert_eq!(active.get(&session.id).unwrap().2, admitted.request_id);
    assert!(!admitted.cancel.is_cancelled());
    drop(active);
    admitted.cancel.cancel();
    streams.0.lock().await.clear();
    session_store::delete_one(&session.id).await.unwrap();
}
