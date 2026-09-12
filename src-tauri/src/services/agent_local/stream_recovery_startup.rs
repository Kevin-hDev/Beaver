pub(crate) async fn recover_all() {
    let Ok(session_ids) = super::stream_recovery_store_discovery::all_session_ids().await else {
        log::warn!("stream_recovery_startup_unavailable");
        return;
    };
    for session_id in session_ids {
        if super::stream_recovery_apply::recover_session(
            &session_id,
            super::stream_recovery_apply::StreamRecoveryMode::StaleOnly,
        )
        .await
        .is_err()
        {
            log::warn!("stream_recovery_session_unavailable session_id={session_id}");
        }
    }
}
