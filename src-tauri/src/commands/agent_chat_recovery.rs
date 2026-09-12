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
    .map_err(public_error)
}

fn public_error(code: String) -> String {
    if code == crate::services::agent_local::session_limits::SESSION_CAPACITY_REACHED {
        code
    } else {
        "conversation_admission_failed".to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn recovery_capacity_keeps_its_public_code() {
        assert_eq!(
            super::public_error("session_capacity_reached".into()),
            "session_capacity_reached"
        );
        assert_eq!(
            super::public_error("private recovery detail".into()),
            "conversation_admission_failed"
        );
    }
}
