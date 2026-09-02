use super::call_context::ExtensionCallContext;
use super::host_identity::HostIdentity;
use serde::Serialize;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const FILE_NAME: &str = "extension-access.jsonl";
const SCHEMA_VERSION: u8 = 1;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AccessResult {
    Granted,
    Denied,
    Failed,
    Revoked,
    Timeout,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry<'a> {
    schema_version: u8,
    date: String,
    identity: &'a str,
    result: AccessResult,
    #[serde(flatten)]
    event: Event<'a>,
}

#[derive(Serialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum Event<'a> {
    CoreCall {
        method: &'a str,
        correlation_id: Uuid,
    },
    HostStarted {
        generation: u64,
        pid: u32,
    },
}

pub(super) fn write_core(context: &ExtensionCallContext, method: &str, result: AccessResult) {
    if write_core_at(&log_path(), context, method, result).is_err() {
        ::log::error!("[extensions] access journal unavailable");
    }
}

pub(super) fn write_core_at(
    path: &Path,
    context: &ExtensionCallContext,
    method: &str,
    result: AccessResult,
) -> Result<(), String> {
    let identity = identity(context.identity());
    let entry = Entry {
        schema_version: SCHEMA_VERSION,
        date: chrono::Utc::now().to_rfc3339(),
        identity: &identity,
        result,
        event: Event::CoreCall {
            method: generic_method(method),
            correlation_id: context.correlation_id(),
        },
    };
    super::bounded_jsonl::write(path, &entry)
}

pub(super) fn write_host_started(identity: HostIdentity, generation: u64, pid: u32) {
    if write_host_started_at(
        &log_path(),
        identity,
        generation,
        pid,
        AccessResult::Granted,
    )
    .is_err()
    {
        ::log::error!("[extensions] access journal unavailable");
    }
}

pub(super) fn write_host_started_at(
    path: &Path,
    identity: HostIdentity,
    generation: u64,
    pid: u32,
    result: AccessResult,
) -> Result<(), String> {
    let identity = identity.label().to_string();
    let entry = Entry {
        schema_version: SCHEMA_VERSION,
        date: chrono::Utc::now().to_rfc3339(),
        identity: &identity,
        result,
        event: Event::HostStarted { generation, pid },
    };
    super::bounded_jsonl::write(path, &entry)
}

fn generic_method(method: &str) -> &str {
    super::types::HOST_TO_CORE_METHODS
        .iter()
        .find(|(name, _, kind, _)| *name == method && *kind == "request")
        .map(|(name, _, _, _)| *name)
        .unwrap_or("unknown")
}

fn identity(identity: &HostIdentity) -> String {
    identity.label().to_string()
}

fn log_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join(FILE_NAME)
}
