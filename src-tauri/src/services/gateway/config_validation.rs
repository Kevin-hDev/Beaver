use std::collections::HashSet;

use crate::models::{ChannelAccountConfig, GatewayConfig};

const MAX_ACCOUNTS_PER_CHANNEL: usize = 16;
const MAX_ALLOWLIST: usize = 100;
const MAX_SESSIONS: u32 = 1_000;
const MAX_MESSAGE_CHARS: u32 = 12_000;
const MAX_RATE: u32 = 10_000;
const MAX_RETENTION_DAYS: u32 = 365;
const MAX_PROVIDER_MODEL: usize = 128;

pub fn validate(config: &GatewayConfig) -> Result<(), String> {
    bounded_nonzero(config.max_sessions, MAX_SESSIONS)?;
    bounded_nonzero(config.message_max_chars, MAX_MESSAGE_CHARS)?;
    bounded_nonzero(config.rate_limits.per_user_per_minute, MAX_RATE)?;
    bounded_nonzero(config.rate_limits.per_channel_per_minute, MAX_RATE)?;
    bounded_nonzero(config.rate_limits.global_per_minute, MAX_RATE)?;
    bounded_nonzero(config.audit.retention_days, MAX_RETENTION_DAYS)?;
    validate_text(&config.default_provider)?;
    reject_interactive_only(&config.default_provider)?;
    validate_text(&config.default_model)?;
    validate_accounts(&config.channels.telegram)?;
    validate_accounts(&config.channels.slack)?;
    validate_accounts(&config.channels.discord)
}

fn bounded_nonzero(value: u32, maximum: u32) -> Result<(), String> {
    if (1..=maximum).contains(&value) {
        Ok(())
    } else {
        Err("configuration Gateway invalide".to_string())
    }
}

fn validate_accounts(accounts: &[ChannelAccountConfig]) -> Result<(), String> {
    if accounts.len() > MAX_ACCOUNTS_PER_CHANNEL {
        return Err("configuration Gateway invalide".to_string());
    }
    let mut account_ids = HashSet::with_capacity(accounts.len());
    for account in accounts {
        super::security::ids::validate_account_id(&account.account_id)?;
        if !account_ids.insert(account.account_id.as_str())
            || account.allowlist.len() > MAX_ALLOWLIST
            || (account.enabled && account.allowlist.is_empty())
        {
            return Err("configuration Gateway invalide".to_string());
        }
        validate_text(&account.provider)?;
        reject_interactive_only(&account.provider)?;
        validate_text(&account.model)?;
        let mut users = HashSet::with_capacity(account.allowlist.len());
        for user in &account.allowlist {
            if user == "*"
                || !users.insert(user.as_str())
                || super::security::ids::validate_external_id(user).is_err()
            {
                return Err("configuration Gateway invalide".to_string());
            }
        }
    }
    Ok(())
}

fn reject_interactive_only(provider: &str) -> Result<(), String> {
    if crate::services::llm::route::is_interactive_only(provider) {
        Err("configuration Gateway invalide".to_string())
    } else {
        Ok(())
    }
}

fn validate_text(value: &str) -> Result<(), String> {
    if value.len() <= MAX_PROVIDER_MODEL && !value.chars().any(char::is_control) {
        Ok(())
    } else {
        Err("configuration Gateway invalide".to_string())
    }
}

#[cfg(test)]
#[path = "config_validation_tests.rs"]
mod tests;
