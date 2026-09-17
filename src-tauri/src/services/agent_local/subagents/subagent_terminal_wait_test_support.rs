static TERMINAL_WAIT_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn lock() -> tokio::sync::MutexGuard<'static, ()> {
    TERMINAL_WAIT_TEST_LOCK.lock().await
}

pub async fn cleanup_parent(parent_id: &str) {
    let report_ids = super::subagent_hidden_reports::peek_reports(parent_id)
        .await
        .into_iter()
        .map(|report| report.id)
        .collect::<Vec<_>>();
    if !report_ids.is_empty() {
        super::subagent_hidden_reports::acknowledge_reports(parent_id, &report_ids)
            .await
            .expect("acknowledge reports during cleanup");
    }
    if let Some(state) = super::subagent_registry::terminal_state_for_parent(parent_id).await {
        let _ = super::subagent_registry::consume_terminal(
            parent_id,
            state.generation,
            state.sequence,
        )
        .await;
    }
    super::session_store::delete_one(parent_id)
        .await
        .expect("delete parent");
}
