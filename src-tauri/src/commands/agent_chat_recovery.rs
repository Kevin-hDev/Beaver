pub(crate) async fn before_admission(
    lease: &crate::services::agent_local::session_locks::AdmissionLease,
    current_execution_id: &str,
) -> Result<(), String> {
    crate::services::agent_local::stream_recovery_apply::recover_session_with_lease(
        lease,
        crate::services::agent_local::stream_recovery_apply::StreamRecoveryMode::Admission {
            current_execution_id,
        },
    )
    .await
    .map_err(|_| "conversation_admission_failed".to_string())
}
