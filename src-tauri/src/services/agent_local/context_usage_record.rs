use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::context_usage_buckets::RequestContextUsage;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum ContextCountSource {
    Provider,
    NativeCounter,
    ModelTokenizer,
    Heuristic,
    Reconstructed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum ContextCountCoverage {
    Complete,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum ContextPreparationState {
    Ready,
    InFlight,
    Completed,
    Stale,
    Interrupted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ContextRequestIdentity {
    pub request_id: String,
    pub turn_id: String,
    pub turn: u32,
    pub attempt: u32,
    pub provider_id: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ContextTokenCount {
    pub tokens: Option<u32>,
    pub capacity_tokens: Option<u32>,
    pub source: Option<ContextCountSource>,
    pub coverage: ContextCountCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ContextPreparationSnapshot {
    pub identity: ContextRequestIdentity,
    pub context_limit: Option<u32>,
    pub input: ContextTokenCount,
    pub state: ContextPreparationState,
    pub breakdown: Option<RequestContextUsage>,
    #[cfg_attr(test, ts(type = "string"))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ContextMeasurementSnapshot {
    pub identity: ContextRequestIdentity,
    pub context_limit: Option<u32>,
    pub input: ContextTokenCount,
    #[cfg_attr(test, ts(type = "string"))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ContextOutputSnapshot {
    pub identity: ContextRequestIdentity,
    pub output: ContextTokenCount,
    #[cfg_attr(test, ts(type = "string"))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ContextUsageRecord {
    pub active_request_id: Option<String>,
    pub current_preparation: Option<ContextPreparationSnapshot>,
    pub last_measurement: Option<ContextMeasurementSnapshot>,
    pub last_output: Option<ContextOutputSnapshot>,
}

impl ContextUsageRecord {
    pub fn invalidate_preparation(&mut self) {
        if let Some(preparation) = &mut self.current_preparation {
            preparation.state = ContextPreparationState::Stale;
            preparation.updated_at = Utc::now();
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self
            .active_request_id
            .as_deref()
            .is_some_and(|id| uuid::Uuid::parse_str(id).is_err())
        {
            return Err(invalid());
        }
        if let Some(snapshot) = &self.current_preparation {
            validate_identity(&snapshot.identity)?;
            validate_count(&snapshot.input)?;
        }
        if let Some(snapshot) = &self.last_measurement {
            validate_identity(&snapshot.identity)?;
            validate_count(&snapshot.input)?;
            if snapshot.input.coverage != ContextCountCoverage::Complete
                || snapshot.input.tokens.is_none()
            {
                return Err(invalid());
            }
        }
        if let Some(snapshot) = &self.last_output {
            validate_identity(&snapshot.identity)?;
            validate_count(&snapshot.output)?;
            if snapshot.output.tokens.is_none() {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

fn validate_identity(identity: &ContextRequestIdentity) -> Result<(), String> {
    if uuid::Uuid::parse_str(&identity.request_id).is_err()
        || uuid::Uuid::parse_str(&identity.turn_id).is_err()
        || identity.attempt == 0
        || !bounded_identifier(&identity.provider_id)
        || !bounded_identifier(&identity.model)
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_count(count: &ContextTokenCount) -> Result<(), String> {
    match count.coverage {
        ContextCountCoverage::Unknown => {
            if count.tokens.is_some() || count.capacity_tokens.is_some() || count.source.is_some() {
                return Err(invalid());
            }
        }
        ContextCountCoverage::Complete | ContextCountCoverage::Partial => {
            if count.tokens.is_none() || count.source.is_none() {
                return Err(invalid());
            }
        }
    }
    if count
        .tokens
        .zip(count.capacity_tokens)
        .is_some_and(|(tokens, capacity)| capacity < tokens)
    {
        return Err(invalid());
    }
    Ok(())
}

fn bounded_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

fn invalid() -> String {
    "context_usage_invalid".into()
}
