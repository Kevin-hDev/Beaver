mod cache;
mod credential_epoch;
mod ledger;
mod ledger_aggregate;
mod pricing;
mod remote;
mod remote_api;
mod remote_codex;
mod remote_gate;
mod remote_kimi_wallet;
mod remote_oauth;
mod remote_parse;
mod request_usage;
mod snapshot;
mod types;

pub use request_usage::RequestUsage;
pub use snapshot::ProviderUsageSnapshot;
pub use types::UsageWorkload;
pub(crate) use types::{origin_for_session, UsageOrigin};

use reqwest::header::HeaderMap;
use tauri::Emitter;

pub async fn snapshot(
    connection_id: &str,
    force_refresh: bool,
) -> Result<ProviderUsageSnapshot, String> {
    types::validate_connection_id(connection_id)?;
    let local = ledger::local_snapshot(connection_id).await;
    let remote = remote::resolve(connection_id, force_refresh).await;
    Ok(snapshot::build_snapshot(connection_id, local, remote))
}

pub fn credential_generation(connection_id: &str) -> Option<u64> {
    credential_epoch::current(connection_id)
}

pub async fn invalidate_remote(connection_id: &str) {
    if types::validate_connection_id(connection_id).is_err() {
        return;
    }
    let _ = credential_epoch::invalidate(connection_id);
    let Some(mut gate) = remote_gate::lock(connection_id).await else {
        return;
    };
    cache::remove(connection_id).await;
    let _ = ledger::clear_remote(connection_id).await;
    remote_gate::reset(&mut gate);
    emit_update(connection_id);
}

pub async fn record_for_session(
    connection_id: &str,
    model: &str,
    session_id: &str,
    workload: UsageWorkload,
    usage: Option<&RequestUsage>,
) {
    let (origin, workload) = types::context_for_session(session_id, workload).await;
    record(connection_id, model, origin, workload, usage).await;
}

pub async fn record_automation(connection_id: &str, model: &str, usage: Option<&RequestUsage>) {
    record(
        connection_id,
        model,
        types::UsageOrigin::Automation,
        UsageWorkload::Primary,
        usage,
    )
    .await;
}

async fn record(
    connection_id: &str,
    model: &str,
    origin: types::UsageOrigin,
    workload: UsageWorkload,
    usage: Option<&RequestUsage>,
) {
    if types::validate_connection_id(connection_id).is_err() {
        return;
    }
    let empty = RequestUsage::default();
    let usage = usage.unwrap_or(&empty);
    let cost = pricing::resolve(connection_id, model, usage).await;
    if ledger::record(connection_id, origin, workload, usage, cost)
        .await
        .is_ok()
    {
        emit_update(connection_id);
    }
}

pub async fn capture_headers(connection_id: &str, generation: Option<u64>, headers: &HeaderMap) {
    let Some(generation) = generation else { return };
    if let Some(remote) = remote_parse::parse_rate_headers(connection_id, headers) {
        let Some(_gate) = remote_gate::lock(connection_id).await else {
            return;
        };
        if !credential_epoch::is_current(connection_id, generation) {
            return;
        }
        let _ = ledger::save_remote(connection_id, generation, remote.clone()).await;
        cache::put(connection_id, generation, remote).await;
        emit_update(connection_id);
    }
}

fn emit_update(connection_id: &str) {
    if let Some(app) = crate::services::agent_local::app_handle_global::get() {
        let _ = app.emit(
            "provider-usage-updated",
            serde_json::json!({ "connectionId": connection_id }),
        );
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod remote_adapter_tests;

#[cfg(test)]
mod remote_kimi_tests;
