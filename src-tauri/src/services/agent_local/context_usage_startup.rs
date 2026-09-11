use chrono::{DateTime, Utc};

use super::context_usage_record::ContextPreparationState;

pub async fn cleanup_interrupted_requests(startup_cutoff: DateTime<Utc>) {
    let Ok(metas) = super::session_index::read_index().await else {
        ::log::warn!("[startup] context usage recovery unavailable");
        return;
    };
    for meta in metas {
        if meta.has_active_context_request
            && meta.updated_at.unwrap_or(meta.created_at) <= startup_cutoff
            && cleanup_one(&meta.id, startup_cutoff).await.is_err()
        {
            ::log::warn!("[startup] context usage recovery skipped");
        }
    }
}

async fn cleanup_one(id: &str, startup_cutoff: DateTime<Utc>) -> Result<bool, String> {
    let lock = super::session_store::lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(id).await?;
    if session.updated_at.unwrap_or(session.created_at) > startup_cutoff
        || session.context_usage.active_request_id.is_none()
    {
        return Ok(false);
    }
    if let Some(preparation) = &mut session.context_usage.current_preparation {
        if preparation.state == ContextPreparationState::InFlight {
            preparation.state = ContextPreparationState::Interrupted;
            preparation.updated_at = Utc::now();
        }
    }
    session.context_usage.active_request_id = None;
    session.updated_at = Some(Utc::now());
    super::session_store::save(&session).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::context_usage_record::{
        ContextCountCoverage, ContextCountSource, ContextPreparationSnapshot,
        ContextRequestIdentity, ContextTokenCount,
    };

    #[tokio::test]
    async fn startup_marks_an_orphaned_in_flight_preparation_interrupted() {
        let mut session = super::super::session_store::create_full(
            "Interrupted context",
            "gpt-5",
            "openai",
            false,
            None,
        )
        .await
        .unwrap();
        let request_id = uuid::Uuid::new_v4().to_string();
        session.context_usage.active_request_id = Some(request_id.clone());
        session.context_usage.current_preparation = Some(ContextPreparationSnapshot {
            identity: ContextRequestIdentity {
                request_id,
                turn_id: uuid::Uuid::new_v4().to_string(),
                turn: 0,
                attempt: 1,
                provider_id: "openai".into(),
                model: "gpt-5".into(),
            },
            context_limit: Some(200_000),
            input: ContextTokenCount {
                tokens: Some(120),
                capacity_tokens: Some(120),
                source: Some(ContextCountSource::Heuristic),
                coverage: ContextCountCoverage::Complete,
            },
            state: ContextPreparationState::InFlight,
            breakdown: None,
            updated_at: Utc::now(),
        });
        super::super::session_store::save(&session).await.unwrap();
        let before = super::super::session_index::read_index()
            .await
            .unwrap()
            .into_iter()
            .find(|meta| meta.id == session.id)
            .unwrap();
        assert!(before.has_active_context_request);

        assert!(cleanup_one(&session.id, Utc::now()).await.unwrap());
        let saved = super::super::session_store::get(&session.id).await.unwrap();
        assert_eq!(saved.context_usage.active_request_id, None);
        assert_eq!(
            saved.context_usage.current_preparation.unwrap().state,
            ContextPreparationState::Interrupted
        );
        let after = super::super::session_index::read_index()
            .await
            .unwrap()
            .into_iter()
            .find(|meta| meta.id == session.id)
            .unwrap();
        assert!(!after.has_active_context_request);
        super::super::session_store::delete_one(&session.id)
            .await
            .unwrap();
    }
}
