use serde::Deserialize;
use zeroize::ZeroizeOnDrop;

#[derive(Debug, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountTokens {
    pub(crate) token: Option<String>,
    pub(crate) bot_token: Option<String>,
    pub(crate) app_token: Option<String>,
}

impl AccountTokens {
    pub fn validate_for(&self, channel_id: &str) -> Result<(), String> {
        match channel_id {
            "telegram" | "discord" => {
                validate_secret(self.token.as_deref())?;
                if self.bot_token.is_some() || self.app_token.is_some() {
                    return Err("identifiants invalides".to_string());
                }
            }
            "slack" => {
                let bot = validate_secret(self.bot_token.as_deref())?;
                let app = validate_secret(self.app_token.as_deref())?;
                if self.token.is_some() || !bot.starts_with("xoxb-") || !app.starts_with("xapp-") {
                    return Err("identifiants invalides".to_string());
                }
            }
            _ => return Err("canal invalide".to_string()),
        }
        Ok(())
    }

    pub(crate) fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub(crate) fn bot_token(&self) -> Option<&str> {
        self.bot_token.as_deref()
    }

    pub(crate) fn app_token(&self) -> Option<&str> {
        self.app_token.as_deref()
    }

    pub fn vault_entries(
        &self,
        channel_id: &str,
        account_id: &str,
    ) -> Result<SecretEntries, String> {
        self.validate_for(channel_id)?;
        let entries = match channel_id {
            "slack" => vec![
                entry(
                    channel_id,
                    account_id,
                    GatewayTokenKind::Bot,
                    required(self.bot_token())?,
                )?,
                entry(
                    channel_id,
                    account_id,
                    GatewayTokenKind::App,
                    required(self.app_token())?,
                )?,
            ],
            "telegram" | "discord" => vec![entry(
                channel_id,
                account_id,
                GatewayTokenKind::Default,
                required(self.token())?,
            )?],
            _ => return Err("canal invalide".to_string()),
        };
        Ok(SecretEntries(entries))
    }
}

fn validate_secret(value: Option<&str>) -> Result<&str, String> {
    let value = value.ok_or_else(|| "identifiants incomplets".to_string())?;
    if value.is_empty()
        || value.len() > 8192
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("identifiants invalides".to_string());
    }
    Ok(value)
}

fn required(value: Option<&str>) -> Result<&str, String> {
    value.ok_or_else(|| "identifiants incomplets".to_string())
}

fn entry(
    channel_id: &str,
    account_id: &str,
    kind: GatewayTokenKind,
    value: &str,
) -> Result<(String, String), String> {
    Ok((
        vault_key(channel_id, account_id, kind)?,
        value.to_string(),
    ))
}

pub struct SecretEntries(Vec<(String, String)>);

impl SecretEntries {
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn as_slice(&self) -> &[(String, String)] {
        &self.0
    }

    fn as_refs(&self) -> Vec<(&str, &str)> {
        self.0
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect()
    }
}

impl Drop for SecretEntries {
    fn drop(&mut self) {
        for (_, value) in &mut self.0 {
            value.zeroize();
        }
    }
}

pub fn store_account_tokens(
    channel_id: &str,
    account_id: &str,
    credentials: &AccountTokens,
) -> Result<(), String> {
    let entries = credentials.vault_entries(channel_id, account_id)?;
    let unchanged = entries.as_slice().iter().all(|(key, value)| {
        api_keys::get_raw(key)
            .map(|current| secrets::constant_time_eq(current.as_bytes(), value.as_bytes()))
            .unwrap_or(false)
    });
    if unchanged {
        return Ok(());
    }
    api_keys::set_raw_batch(&entries.as_refs())
}
