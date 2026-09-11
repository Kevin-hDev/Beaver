use super::*;
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

pub async fn list(actor: &AutomationActor) -> Result<Vec<AutomationSummary>, AutomationError> {
    let paused = crate::services::config::read_config()
        .map_err(|_| AutomationError::StoreUnavailable)?
        .heartbeat
        .global_paused;
    list_with_pause_at(
        &crate::services::paths::data_dir(),
        actor,
        Utc::now(),
        paused,
    )
    .await
}

pub async fn get(actor: &AutomationActor, id: Uuid) -> Result<AutomationDetail, AutomationError> {
    get_at(&crate::services::paths::data_dir(), actor, id, Utc::now()).await
}

pub async fn create(
    actor: &AutomationActor,
    input: CreateAutomation,
) -> Result<AutomationDetail, AutomationError> {
    super::service_mutations::create_at(
        &crate::services::paths::data_dir(),
        actor,
        input,
        Utc::now(),
    )
    .await
}

pub async fn update(
    actor: &AutomationActor,
    id: Uuid,
    patch: UpdateAutomation,
) -> Result<AutomationDetail, AutomationError> {
    super::service_mutations::update_at(
        &crate::services::paths::data_dir(),
        actor,
        id,
        patch,
        Utc::now(),
    )
    .await
}

pub async fn delete(actor: &AutomationActor, id: Uuid) -> Result<(), AutomationError> {
    super::service_mutations::delete_at(&crate::services::paths::data_dir(), actor, id).await
}

pub async fn history(
    actor: &AutomationActor,
    query: HistoryQuery,
) -> Result<HistoryPage, AutomationError> {
    history_at(&crate::services::paths::data_dir(), actor, query).await
}

pub async fn disable_missing_target(id: Uuid) -> Result<(), AutomationError> {
    super::service_mutations::disable_missing_target_at(&crate::services::paths::data_dir(), id)
        .await
}

#[cfg(test)]
pub(crate) async fn list_at(
    root: &Path,
    actor: &AutomationActor,
    now: DateTime<Utc>,
) -> Result<Vec<AutomationSummary>, AutomationError> {
    list_with_pause_at(root, actor, now, false).await
}

async fn list_with_pause_at(
    root: &Path,
    actor: &AutomationActor,
    now: DateTime<Utc>,
    globally_paused: bool,
) -> Result<Vec<AutomationSummary>, AutomationError> {
    let operation = super::audit_store::begin(root, actor, "list", None, Vec::new()).await?;
    let result = async {
        let definitions = super::store::read_all_at(root)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        let runs = super::history_store::all_at(&root.join("logs/wakeups.jsonl"), None)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        let runtime = match super::runtime_lifecycle::runtime_at(root).await {
            Ok(runtime) => Some(runtime),
            Err(AutomationError::NotFound) => None,
            Err(_) => return Err(AutomationError::StoreUnavailable),
        };
        Ok(definitions
            .into_iter()
            .map(|definition| {
                let last_run = runs
                    .iter()
                    .find(|run| run.automation_id == definition.id.to_string())
                    .map(|run| AutomationLastRun {
                        status: run.status.clone(),
                        finished_at: run.finished_at.clone(),
                        error_code: run.error_code,
                    });
                let paused =
                    globally_paused && definition.status == crate::models::AutomationStatus::Active;
                let running = runtime.as_ref().is_some_and(|runtime| {
                    runtime.occurrences.iter().any(|occurrence| {
                        occurrence.automation_id == definition.id
                            && occurrence.state == super::OccurrenceState::Running
                    })
                });
                super::service_helpers::summary_with_state(
                    definition, now, paused, running, last_run,
                )
            })
            .collect::<Vec<_>>())
    }
    .await;
    let count = result.as_ref().ok().map(Vec::len);
    super::audit_store::complete(root, actor, operation, "list", None, result, count).await
}

pub(crate) async fn get_at(
    root: &Path,
    actor: &AutomationActor,
    id: Uuid,
    now: DateTime<Utc>,
) -> Result<AutomationDetail, AutomationError> {
    let operation = super::audit_store::begin(root, actor, "get", Some(id), Vec::new()).await?;
    let result = match super::service_helpers::find(root, id).await {
        Ok(definition) => super::service_helpers::detail(definition, now),
        Err(error) => Err(error),
    };
    super::audit_store::complete(root, actor, operation, "get", Some(id), result, None).await
}

pub(crate) async fn history_at(
    root: &Path,
    actor: &AutomationActor,
    query: HistoryQuery,
) -> Result<HistoryPage, AutomationError> {
    let operation = super::audit_store::begin(
        root,
        actor,
        "history",
        Some(query.automation_id),
        Vec::new(),
    )
    .await?;
    let path = root.join("logs").join("wakeups.jsonl");
    let result = super::history_store::page_at(
        &path,
        query.automation_id,
        query.limit,
        query.cursor.as_deref(),
    )
    .await;
    let count = result.as_ref().ok().map(|page| page.entries.len());
    super::audit_store::complete(
        root,
        actor,
        operation,
        "history",
        Some(query.automation_id),
        result,
        count,
    )
    .await
}

#[cfg(test)]
pub(crate) use super::service_mutations::{create_at, delete_at, update_at};
