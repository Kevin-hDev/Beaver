use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use super::{qwen_endpoints, workspace_id};

pub const VAULT_KEY: &str = "provider_connection:qwen";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum QwenRegion {
    Beijing,
    Singapore,
    HongKong,
    Tokyo,
    Frankfurt,
    Virginia,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum QwenEndpointMode {
    Shared,
    Workspace,
    Trial,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct QwenConnectionInput {
    pub region: QwenRegion,
    pub endpoint_mode: QwenEndpointMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub workspace_id: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QwenBillingPlan {
    PayAsYouGo,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct QwenConnectionRecord {
    pub schema_version: u16,
    pub billing_plan: QwenBillingPlan,
    pub connection: QwenConnectionInput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QwenResolvedEndpoint {
    pub base_url: String,
    pub models_url: String,
}

pub fn validate_qwen_connection(input: &QwenConnectionInput) -> Result<(), &'static str> {
    match (input.endpoint_mode, input.workspace_id.as_deref()) {
        (QwenEndpointMode::Workspace, Some(workspace)) => workspace_id::validate(workspace)?,
        (QwenEndpointMode::Workspace, None) => return Err("provider_configuration_invalid"),
        (_, Some(_)) => return Err("provider_configuration_invalid"),
        (_, None) => {}
    }
    qwen_endpoints::base_url(
        input.region,
        input.endpoint_mode,
        input.workspace_id.as_deref(),
    )
    .ok_or("provider_configuration_invalid")?;
    Ok(())
}

pub fn resolve_qwen_endpoint(
    input: &QwenConnectionInput,
) -> Result<QwenResolvedEndpoint, &'static str> {
    validate_qwen_connection(input)?;
    let base_url = qwen_endpoints::base_url(
        input.region,
        input.endpoint_mode,
        input.workspace_id.as_deref(),
    )
    .ok_or("provider_configuration_invalid")?;
    Ok(QwenResolvedEndpoint {
        models_url: format!("{base_url}/models"),
        base_url,
    })
}

pub fn encode(input: QwenConnectionInput) -> Result<Zeroizing<String>, String> {
    validate_qwen_connection(&input).map_err(str::to_string)?;
    serde_json::to_string(&QwenConnectionRecord {
        schema_version: 1,
        billing_plan: QwenBillingPlan::PayAsYouGo,
        connection: input,
    })
    .map(Zeroizing::new)
    .map_err(|_| "provider_configuration_invalid".to_string())
}

pub fn decode(value: &str) -> Result<QwenConnectionRecord, String> {
    let record: QwenConnectionRecord =
        serde_json::from_str(value).map_err(|_| "provider_configuration_invalid".to_string())?;
    if record.schema_version != 1 || record.billing_plan != QwenBillingPlan::PayAsYouGo {
        return Err("provider_configuration_invalid".to_string());
    }
    validate_qwen_connection(&record.connection).map_err(str::to_string)?;
    Ok(record)
}

pub fn load() -> Result<QwenConnectionRecord, String> {
    let raw = crate::services::api_keys::get_raw(VAULT_KEY)?;
    decode(&raw)
}

pub fn load_resolved_endpoint() -> Result<Option<QwenResolvedEndpoint>, String> {
    if !crate::services::api_keys::has_raw(VAULT_KEY)? {
        return Ok(None);
    }
    let record = load()?;
    resolve_qwen_endpoint(&record.connection)
        .map(Some)
        .map_err(str::to_string)
}
