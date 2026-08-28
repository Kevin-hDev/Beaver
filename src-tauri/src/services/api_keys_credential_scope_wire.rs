use serde::{Deserialize, Serialize};

use crate::services::reasoning_continuity::contract::{CredentialScope, RouteId};

const OAUTH_CREDENTIAL_SCHEMA_VERSION: u16 = 1;
pub(crate) const LLM_OAUTH_XAI_KEY: &str = "_llm_oauth_xai";
pub(crate) const LLM_OAUTH_KIMI_KEY: &str = "_llm_oauth_kimi";
pub(crate) const CODEX_OAUTH_KEY: &str = "_codex_oauth";

#[derive(Serialize, Deserialize)]
pub(crate) struct LlmOAuthCredentialRecord {
    #[serde(default)]
    pub schema_version: u16,
    pub access: String,
    pub refresh: String,
    pub expires_at: i64,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub credential_scope: Option<CredentialScope>,
}

impl Drop for LlmOAuthCredentialRecord {
    fn drop(&mut self) {
        self.access.zeroize();
        self.refresh.zeroize();
        if let Some(value) = &mut self.user_id {
            value.zeroize();
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CodexOAuthCredentialRecord {
    #[serde(default)]
    pub schema_version: u16,
    pub access: String,
    pub refresh: String,
    pub expires_at: i64,
    #[serde(default)]
    pub refresh_not_before: i64,
    #[serde(rename = "account_id")]
    pub account_hint: String,
    #[serde(default)]
    pub credential_scope: Option<CredentialScope>,
}

impl Drop for CodexOAuthCredentialRecord {
    fn drop(&mut self) {
        self.access.zeroize();
        self.refresh.zeroize();
        self.account_hint.zeroize();
    }
}

pub(crate) fn new_llm_oauth_record(
    access: String,
    refresh: String,
    expires_at: i64,
    user_id: Option<String>,
    credential_scope: CredentialScope,
) -> LlmOAuthCredentialRecord {
    LlmOAuthCredentialRecord {
        schema_version: OAUTH_CREDENTIAL_SCHEMA_VERSION,
        access,
        refresh,
        expires_at,
        user_id,
        credential_scope: Some(credential_scope),
    }
}

pub(crate) fn new_codex_oauth_record(
    access: String,
    refresh: String,
    expires_at: i64,
    refresh_not_before: i64,
    account_hint: String,
    credential_scope: CredentialScope,
) -> CodexOAuthCredentialRecord {
    CodexOAuthCredentialRecord {
        schema_version: OAUTH_CREDENTIAL_SCHEMA_VERSION,
        access,
        refresh,
        expires_at,
        refresh_not_before,
        account_hint,
        credential_scope: Some(credential_scope),
    }
}

pub(crate) fn decode_llm_oauth_record(
    json: &str,
    route: RouteId,
) -> Result<LlmOAuthCredentialRecord, String> {
    let record: LlmOAuthCredentialRecord =
        serde_json::from_str(json).map_err(|_| credential_error())?;
    validate_record_version(
        record.schema_version,
        record.credential_scope.as_ref(),
        route,
    )?;
    Ok(record)
}

pub(crate) fn decode_codex_oauth_record(json: &str) -> Result<CodexOAuthCredentialRecord, String> {
    let record: CodexOAuthCredentialRecord =
        serde_json::from_str(json).map_err(|_| credential_error())?;
    validate_record_version(
        record.schema_version,
        record.credential_scope.as_ref(),
        RouteId::CodexOauth,
    )?;
    Ok(record)
}

pub(crate) fn encode_llm_oauth_record(
    record: &LlmOAuthCredentialRecord,
    route: RouteId,
) -> Result<Zeroizing<String>, String> {
    validate_current_record(
        record.schema_version,
        record.credential_scope.as_ref(),
        route,
    )?;
    serde_json::to_string(record)
        .map(Zeroizing::new)
        .map_err(|_| credential_error())
}

pub(crate) fn encode_codex_oauth_record(
    record: &CodexOAuthCredentialRecord,
) -> Result<Zeroizing<String>, String> {
    validate_current_record(
        record.schema_version,
        record.credential_scope.as_ref(),
        RouteId::CodexOauth,
    )?;
    serde_json::to_string(record)
        .map(Zeroizing::new)
        .map_err(|_| credential_error())
}

fn validate_record_version(
    version: u16,
    scope: Option<&CredentialScope>,
    route: RouteId,
) -> Result<(), String> {
    match (version, scope) {
        (0, None) => Ok(()),
        (OAUTH_CREDENTIAL_SCHEMA_VERSION, Some(scope)) => scope
            .validate_for_route(route)
            .map_err(|_| credential_error()),
        _ => Err(credential_error()),
    }
}

fn validate_current_record(
    version: u16,
    scope: Option<&CredentialScope>,
    route: RouteId,
) -> Result<(), String> {
    if version != OAUTH_CREDENTIAL_SCHEMA_VERSION {
        return Err(credential_error());
    }
    scope
        .ok_or_else(credential_error)?
        .validate_for_route(route)
        .map_err(|_| credential_error())
}

fn credential_error() -> String {
    "provider_configuration_invalid".to_string()
}
