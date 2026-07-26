use zeroize::Zeroize;

use crate::services::api_keys;
use crate::services::gateway::security::{ids, secrets};

include!("tokens_account.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayTokenKind {
    Default,
    Bot,
    App,
}

impl GatewayTokenKind {
    pub fn parse(channel_id: &str, raw: &str) -> Result<Self, String> {
        match (channel_id, raw) {
            ("telegram" | "discord", "default") => Ok(Self::Default),
            ("slack", "bot") => Ok(Self::Bot),
            ("slack", "app") => Ok(Self::App),
            _ => Err("type de token invalide".to_string()),
        }
    }

    fn suffix(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Bot => ".bot",
            Self::App => ".app",
        }
    }
}

pub fn vault_key(
    channel_id: &str,
    account_id: &str,
    kind: GatewayTokenKind,
) -> Result<String, String> {
    ids::validate_channel_id(channel_id)?;
    ids::validate_account_id(account_id)?;
    Ok(format!(
        "gateway.{channel_id}.{account_id}{}",
        kind.suffix()
    ))
}

pub fn has(channel_id: &str, account_id: &str, token_kind: &str) -> Result<bool, String> {
    let kind = GatewayTokenKind::parse(channel_id, token_kind)?;
    let key = vault_key(channel_id, account_id, kind)?;
    Ok(api_keys::has_key(&format!("raw:{key}")))
}

pub fn delete(channel_id: &str, account_id: &str, token_kind: Option<&str>) -> Result<(), String> {
    match token_kind {
        Some(kind) => {
            let kind = GatewayTokenKind::parse(channel_id, kind)?;
            api_keys::delete_raw(&vault_key(channel_id, account_id, kind)?)
        }
        None if channel_id == "slack" => {
            let bot = vault_key(channel_id, account_id, GatewayTokenKind::Bot)?;
            let app = vault_key(channel_id, account_id, GatewayTokenKind::App)?;
            api_keys::delete_raw_batch(&[bot.as_str(), app.as_str()])
        }
        None => api_keys::delete_raw(&vault_key(
            channel_id,
            account_id,
            GatewayTokenKind::Default,
        )?),
    }
}

pub fn required_kinds(channel_id: &str) -> &'static [&'static str] {
    match channel_id {
        "slack" => &["bot", "app"],
        "telegram" | "discord" => &["default"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_channel_token_kinds() {
        assert_eq!(
            GatewayTokenKind::parse("telegram", "default").unwrap(),
            GatewayTokenKind::Default
        );
        assert_eq!(
            GatewayTokenKind::parse("discord", "default").unwrap(),
            GatewayTokenKind::Default
        );
        assert_eq!(
            GatewayTokenKind::parse("slack", "bot").unwrap(),
            GatewayTokenKind::Bot
        );
        assert_eq!(
            GatewayTokenKind::parse("slack", "app").unwrap(),
            GatewayTokenKind::App
        );
    }

    #[test]
    fn rejects_wrong_token_kind_for_channel() {
        assert!(GatewayTokenKind::parse("telegram", "bot").is_err());
        assert!(GatewayTokenKind::parse("discord", "app").is_err());
        assert!(GatewayTokenKind::parse("slack", "default").is_err());
    }

    #[test]
    fn builds_valid_vault_keys() {
        assert_eq!(
            vault_key("telegram", "bot-main", GatewayTokenKind::Default).unwrap(),
            "gateway.telegram.bot-main"
        );
        assert_eq!(
            vault_key("slack", "workspace", GatewayTokenKind::Bot).unwrap(),
            "gateway.slack.workspace.bot"
        );
        assert_eq!(
            vault_key("slack", "workspace", GatewayTokenKind::App).unwrap(),
            "gateway.slack.workspace.app"
        );
    }

    #[test]
    fn rejects_path_like_account_id() {
        assert!(vault_key("telegram", "../secret", GatewayTokenKind::Default).is_err());
    }
}

#[cfg(test)]
#[path = "tokens_account_tests.rs"]
mod account_tests;
