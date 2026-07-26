use super::channels::GatewayError;
use super::security::audit::{self, AuditAction};
use super::types::ChannelKey;

pub(crate) fn invalid_account_config(
    channel_id: &str,
    account_id: &str,
    message: &str,
) -> Result<(), String> {
    log(
        channel_id,
        account_id,
        AuditAction::Blocked,
        Some("invalid_config"),
        Some(message),
    )
}

pub(crate) fn auth_error(key: &ChannelKey, error: &GatewayError) -> Result<(), String> {
    if !error.is_auth {
        return Ok(());
    }
    log(
        &key.channel_id,
        &key.account_id,
        AuditAction::AuthFailed,
        None,
        Some(&error.message),
    )
}

pub(crate) fn channel_started(key: &ChannelKey) -> Result<(), String> {
    log(
        &key.channel_id,
        &key.account_id,
        AuditAction::ChannelStarted,
        None,
        None,
    )
}

pub(crate) fn channel_stopped(
    key: &ChannelKey,
    decision: Option<&str>,
    error: Option<&str>,
) -> Result<(), String> {
    log(
        &key.channel_id,
        &key.account_id,
        AuditAction::ChannelStopped,
        decision,
        error,
    )
}

fn log(
    channel_id: &str,
    account_id: &str,
    action: AuditAction,
    decision: Option<&str>,
    error: Option<&str>,
) -> Result<(), String> {
    let safe_error = error.map(audit::sanitize_error);
    audit::log_gateway_action(
        channel_id,
        account_id,
        "",
        action,
        decision,
        safe_error.as_deref(),
    )
}
