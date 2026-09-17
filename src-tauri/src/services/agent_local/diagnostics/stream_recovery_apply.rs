use super::conversation_interrupted_tail::RecoveryProof;
use super::session_locks::AdmissionLease;

pub(crate) enum StreamRecoveryMode<'a> {
    Owner {
        request_id: &'a str,
        terminal: OwnerTerminal<'a>,
    },
    Admission {
        current_execution_id: &'a str,
        resume_message_id: Option<&'a str>,
    },
    StaleOnly,
}

#[derive(Clone, Copy)]
pub(crate) enum OwnerTerminal<'a> {
    Cancelled,
    Failed { code: &'a str },
}

pub(crate) async fn recover_session(
    session_id: &str,
    mode: StreamRecoveryMode<'_>,
) -> Result<(), String> {
    let lease = super::session_locks::acquire_admission_lease(session_id).await;
    recover_session_with_lease(&lease, mode).await
}

pub(crate) async fn recover_session_with_lease(
    lease: &AdmissionLease,
    mode: StreamRecoveryMode<'_>,
) -> Result<(), String> {
    let paths = super::stream_recovery_store_discovery::session_paths(lease.session_id()).await?;
    let mut session = match super::session_store::get(lease.session_id()).await {
        Ok(session) => session,
        Err(error) if paths.is_empty() => return Err(error),
        Err(_) if !super::session_store::document_exists(lease.session_id()).await => {
            for path in paths {
                super::stream_recovery_store::remove(path).await?;
            }
            return Ok(());
        }
        Err(_) => return Err(error()),
    };
    let mut projections = Vec::new();
    for path in paths {
        let projection = match super::stream_recovery_projection::from_path(&path) {
            Ok(projection) => projection,
            Err(_) => {
                super::stream_recovery_store_discovery::quarantine(path).await?;
                continue;
            }
        };
        if !super::stream_recovery_apply_validation::path_matches(
            lease.session_id(),
            &path,
            &projection,
        ) {
            super::stream_recovery_store_discovery::quarantine(path).await?;
            continue;
        }
        if super::stream_recovery_apply_validation::should_skip_live(&projection, &mode) {
            continue;
        }
        projections.push((projection.header.created_at, path));
    }
    projections.sort_by_key(|(created_at, _)| *created_at);

    let mut claimed = Vec::new();
    let mut changed = false;
    let mut recovered_any = false;
    for (_, path) in projections {
        let path = super::stream_recovery_store_discovery::claim(path).await?;
        let Some(projection) = load_claimed(&path).await? else {
            continue;
        };
        if super::stream_recovery_apply_validation::superseded(&session, &projection)
            || !super::stream_recovery_apply_validation::owner_matches(&session, &projection)
        {
            claimed.push(path);
            continue;
        }
        changed |= super::conversation_interrupted_tail::apply_recovered_projection(
            &mut session,
            &projection,
        )
        .map_err(super::stream_recovery_apply_validation::map_tail_error)?;
        changed |= super::conversation_interrupted_tail::close_recoverable(
            &mut session,
            RecoveryProof::RecoveredJournal {
                request_id: &projection.header.request_id,
            },
        )
        .map_err(super::stream_recovery_apply_validation::map_tail_error)?;
        changed |= super::stream_recovery_apply_validation::mark_terminal(
            &mut session,
            &projection.header.request_id,
            &mode,
        );
        changed |= super::context_usage_startup::mark_interrupted(
            &mut session,
            Some(&projection.header.request_id),
        );
        recovered_any = true;
        claimed.push(path);
    }

    if !recovered_any {
        if let StreamRecoveryMode::Admission {
            current_execution_id,
            resume_message_id,
        } = mode
        {
            // An explicit Resume owns this user-only tail. Closing it here would
            // make conversation_resume reject the message immediately after.
            let resumes_current_tail = resume_message_id.is_some_and(|message_id| {
                session
                    .messages
                    .last()
                    .is_some_and(|message| message.role == "user" && message.id == message_id)
            });
            if !resumes_current_tail {
                changed |= super::conversation_interrupted_tail::close_recoverable(
                    &mut session,
                    RecoveryProof::AdmissionFallback {
                        current_execution_id,
                    },
                )
                .map_err(super::stream_recovery_apply_validation::map_tail_error)?;
            }
        }
    }
    if changed {
        super::session_store_messages::recompute_accumulated_tokens(&mut session);
        session.context_usage.invalidate_preparation();
        session.updated_at = Some(chrono::Utc::now());
        super::conversation_history_validation::validate(&session.messages).map_err(|_| error())?;
        super::session_store::save(&session)
            .await
            .map_err(|_| error())?;
    }
    for path in claimed {
        if super::stream_recovery_store::remove(path).await.is_err() {
            log::warn!("stream_recovery_cleanup_failed");
        }
    }
    Ok(())
}

pub(super) async fn load_claimed(
    path: &std::path::Path,
) -> Result<Option<super::stream_recovery_projection::RecoveryProjection>, String> {
    match super::stream_recovery_projection::from_path(path) {
        Ok(projection) => Ok(Some(projection)),
        Err(_) => {
            super::stream_recovery_store_discovery::quarantine(path.to_path_buf()).await?;
            Ok(None)
        }
    }
}

#[cfg(test)]
pub(crate) fn close_admission_fallback(
    session: &mut super::types_session::AgentSession,
    current_execution_id: Option<&str>,
) -> Result<bool, String> {
    let proof = current_execution_id.map_or(
        RecoveryProof::LegacyWithoutExecution,
        |current_execution_id| RecoveryProof::AdmissionFallback {
            current_execution_id,
        },
    );
    super::conversation_interrupted_tail::close_recoverable(session, proof)
        .map_err(super::stream_recovery_apply_validation::map_tail_error)
}

fn error() -> String {
    "stream_recovery_unavailable".into()
}
