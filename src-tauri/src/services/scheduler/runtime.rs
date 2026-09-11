use super::work_supervision::SchedulerWakeupWork;
use crate::models::{AutomationDefinition, AutomationStatus};
use crate::services::automations::{
    AutomationError, AutomationRuntime, OccurrenceResult, OccurrenceState,
};
use crate::services::work_registry::ServiceWorkCancellation;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;
use std::path::Path;
use tauri::AppHandle;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub type AutomationRunResult = OccurrenceResult;

pub(super) async fn run_loop(
    app: AppHandle,
    mut reload_rx: watch::Receiver<u64>,
    lifetime: ServiceWorkCancellation,
    work: SchedulerWakeupWork,
) {
    recover_and_publish().await;
    loop {
        if lifetime.is_cancelled() {
            return;
        }
        let sleep = tick(&app, &lifetime, &work).await;
        tokio::select! {
            _ = lifetime.cancelled() => return,
            _ = tokio::time::sleep(sleep) => {},
            _ = reload_rx.changed() => {},
        }
    }
}

async fn recover_and_publish() {
    match crate::services::automations::recover_startup(Utc::now()).await {
        Ok(ids) => publish_all(ids).await,
        Err(_) => ::log::warn!("[scheduler] reprise des automatisations indisponible"),
    }
}

async fn tick(
    app: &AppHandle,
    lifetime: &ServiceWorkCancellation,
    work: &SchedulerWakeupWork,
) -> std::time::Duration {
    let definitions = match crate::services::automations::read_all().await {
        Ok(items) => items,
        Err(_) => {
            ::log::warn!("[scheduler] définitions d'automatisation indisponibles");
            return std::time::Duration::from_secs(60);
        }
    };
    let paused = crate::services::config::read_config()
        .map(|config| config.heartbeat.global_paused)
        .unwrap_or(true);
    let now = Utc::now();
    match scan_if_active_at(
        &crate::services::paths::data_dir(),
        paused,
        now,
        &definitions,
    )
    .await
    {
        Ok(_) => {}
        Err(_) => {
            ::log::warn!("[scheduler] balayage des automatisations indisponible");
            return std::time::Duration::from_secs(60);
        }
    }
    let runtime = match crate::services::automations::read_runtime().await {
        Ok(runtime) => runtime,
        Err(_) => {
            ::log::warn!("[scheduler] runtime des automatisations indisponible");
            return std::time::Duration::from_secs(60);
        }
    };
    publish_all(terminal_ids(&runtime)).await;
    if !paused {
        launch_ready(app, lifetime, work, &definitions).await;
    }
    let has_pending = runtime
        .occurrences
        .iter()
        .any(|item| item.state == OccurrenceState::Pending);
    sleep_until_next(&definitions, paused, now, has_pending)
}

pub(crate) async fn scan_if_active_at(
    root: &Path,
    paused: bool,
    now: DateTime<Utc>,
    definitions: &[AutomationDefinition],
) -> Result<Vec<Uuid>, String> {
    if paused {
        return Ok(Vec::new());
    }
    crate::services::automations::scan_and_advance_at(root, now, definitions).await
}

pub(crate) fn terminal_ids(runtime: &AutomationRuntime) -> Vec<Uuid> {
    runtime
        .occurrences
        .iter()
        .filter(|item| item.state == OccurrenceState::Terminal)
        .map(|item| item.id)
        .collect()
}

async fn publish_all(ids: Vec<Uuid>) {
    for id in ids {
        if publish_terminal(id).await.is_err() {
            ::log::warn!("[scheduler] publication terminale différée");
        }
    }
}

async fn launch_ready(
    app: &AppHandle,
    lifetime: &ServiceWorkCancellation,
    work: &SchedulerWakeupWork,
    definitions: &[AutomationDefinition],
) {
    let Ok(runtime) = crate::services::automations::read_runtime().await else {
        return;
    };
    for (automation_id, occurrence_id) in ready(&runtime, definitions) {
        if lifetime.is_cancelled() {
            return;
        }
        let app = app.clone();
        let result = work.spawn(move |service_cancel| async move {
            let cancel = CancellationToken::new();
            let shutdown = cancel.clone();
            let run = super::fire::fire_automation(app, automation_id, occurrence_id, cancel);
            tokio::pin!(run);
            tokio::select! {
                biased;
                _ = service_cancel.cancelled() => { shutdown.cancel(); run.await; }
                _ = &mut run => {}
            }
        });
        if result.is_err() {
            ::log::warn!("[scheduler] capacité d'exécution indisponible");
            return;
        }
    }
}

fn ready(runtime: &AutomationRuntime, definitions: &[AutomationDefinition]) -> Vec<(Uuid, Uuid)> {
    let active = definitions
        .iter()
        .filter(|item| item.status == AutomationStatus::Active)
        .map(|item| item.id)
        .collect::<HashSet<_>>();
    let running = runtime
        .occurrences
        .iter()
        .filter(|item| item.state == OccurrenceState::Running)
        .map(|item| item.automation_id)
        .collect::<HashSet<_>>();
    runtime
        .occurrences
        .iter()
        .filter(|item| {
            item.state == OccurrenceState::Pending
                && active.contains(&item.automation_id)
                && !running.contains(&item.automation_id)
        })
        .map(|item| (item.automation_id, item.id))
        .collect()
}

pub(crate) fn sleep_until_next(
    definitions: &[AutomationDefinition],
    paused: bool,
    now: DateTime<Utc>,
    has_pending: bool,
) -> std::time::Duration {
    if paused || has_pending {
        return std::time::Duration::from_secs(60);
    }
    let cap = now + Duration::minutes(60);
    let next = definitions
        .iter()
        .filter_map(|item| {
            crate::services::automations::next_fire::next_fire_at(item, now)
                .ok()
                .flatten()
        })
        .map(|item| item.at)
        .min()
        .unwrap_or(cap)
        .min(cap);
    (next - now)
        .to_std()
        .unwrap_or(std::time::Duration::from_secs(1))
}

pub async fn mark_running(id: Uuid, started_at: DateTime<Utc>) -> Result<(), AutomationError> {
    crate::services::automations::mark_runtime_running_at(
        &crate::services::paths::data_dir(),
        id,
        started_at,
    )
    .await
}

pub async fn mark_terminal(id: Uuid, result: AutomationRunResult) -> Result<(), AutomationError> {
    crate::services::automations::mark_runtime_terminal_at(
        &crate::services::paths::data_dir(),
        id,
        result,
    )
    .await
}

pub async fn publish_terminal(id: Uuid) -> Result<(), AutomationError> {
    super::runtime_publish::publish_terminal_at(&crate::services::paths::data_dir(), id).await
}
