use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::models::AuditConfig;

const HMAC_VAULT_KEY: &str = "gateway.audit.hmac.v1";
static ENABLED: AtomicBool = AtomicBool::new(true);
static RETENTION_DAYS: AtomicU32 = AtomicU32::new(30);

#[derive(Debug, Deserialize, Serialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub channel: String,
    pub account_id: String,
    pub user_hash: String,
    pub action: AuditAction,
    pub security_decision: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    MessageReceived,
    MessageSent,
    Blocked,
    RateLimited,
    AgentError,
    ChannelStarted,
    ChannelStopped,
    AuthFailed,
}

pub fn configure(config: &AuditConfig) {
    ENABLED.store(config.enabled, Ordering::Relaxed);
    RETENTION_DAYS.store(config.retention_days.clamp(1, 365), Ordering::Relaxed);
}

pub fn hash_user_id(user_id: &str) -> Result<String, String> {
    if user_id.len() > 128 || user_id.chars().any(char::is_control) {
        return Err("identité d'audit invalide".to_string());
    }
    let key = crate::services::api_keys::get_or_create_random_raw(HMAC_VAULT_KEY, 32)?;
    hash_user_id_with_key(user_id, &key)
}

fn hash_user_id_with_key(user_id: &str, key: &[u8]) -> Result<String, String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|_| "identité d'audit indisponible".to_string())?;
    mac.update(user_id.as_bytes());
    let digest = mac.finalize().into_bytes();
    Ok(hex::encode(&digest[..8]))
}

pub fn log_audit(entry: &AuditEntry) -> Result<(), String> {
    if !ENABLED.load(Ordering::Relaxed) {
        return Ok(());
    }
    log_audit_to_path(entry, &audit_path())
}

fn log_audit_to_path(entry: &AuditEntry, path: &std::path::Path) -> Result<(), String> {
    let line =
        serde_json::to_string(entry).map_err(|_| "journal d'audit indisponible".to_string())?;
    let retention = RETENTION_DAYS.load(Ordering::Relaxed);
    append_serialized(path, &line, retention)
}

pub fn log_gateway_action(
    channel: &str,
    account_id: &str,
    user_id: &str,
    action: AuditAction,
    security_decision: Option<&str>,
    error: Option<&str>,
) -> Result<(), String> {
    let user_hash = hash_user_id(user_id)?;
    log_audit(&AuditEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        channel: channel.to_string(),
        account_id: account_id.to_string(),
        user_hash,
        action,
        security_decision: security_decision.map(safe_code),
        error: error.map(sanitize_error),
    })
}

pub fn sanitize_error(_error: &str) -> String {
    "operation_failed".to_string()
}

fn safe_code(value: &str) -> String {
    if !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
    {
        value.to_string()
    } else {
        "blocked".to_string()
    }
}

fn audit_path() -> PathBuf {
    crate::services::paths::data_dir().join("logs/gateway-audit.jsonl")
}

include!("audit_store.rs");

#[cfg(test)]
#[path = "audit_tests.rs"]
mod tests;
