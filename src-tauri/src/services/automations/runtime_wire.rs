use super::runtime_store::{
    AutomationOccurrence, AutomationRuntime, OccurrenceResult, OccurrenceResultStatus,
    OccurrenceState, AUTOMATION_RUNTIME_SCHEMA_VERSION,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct RuntimeWire {
    schema_version: u32,
    last_checked_at: DateTime<Utc>,
    occurrences: Vec<OccurrenceWire>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum OccurrenceWire {
    Pending {
        #[serde(flatten)]
        common: CommonWire,
        coalesced_count: u32,
        last_scheduled_for: DateTime<Utc>,
    },
    Running {
        #[serde(flatten)]
        common: CommonWire,
        started_at: DateTime<Utc>,
    },
    Terminal {
        #[serde(flatten)]
        common: CommonWire,
        started_at: Option<DateTime<Utc>>,
        result: ResultWire,
    },
}

#[derive(Serialize, Deserialize)]
struct CommonWire {
    id: Uuid,
    automation_id: Uuid,
    scheduled_for: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct ResultWire {
    status: ResultStatusWire,
    finished_at: DateTime<Utc>,
    error_code: Option<String>,
    session_id: Option<String>,
    tokens: Option<u32>,
    missed_count: Option<u32>,
    first_scheduled_for: Option<DateTime<Utc>>,
    last_scheduled_for: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ResultStatusWire {
    Ok,
    Error,
    Missed,
    Cancelled,
    Interrupted,
}

pub(super) fn encode(runtime: &AutomationRuntime) -> Result<Vec<u8>, String> {
    let wire = RuntimeWire {
        schema_version: runtime.schema_version,
        last_checked_at: runtime.last_checked_at,
        occurrences: runtime
            .occurrences
            .iter()
            .map(to_wire)
            .collect::<Result<_, _>>()?,
    };
    serde_json::to_vec_pretty(&wire).map_err(|_| super::runtime_store::runtime_error())
}

pub(super) fn decode(bytes: &[u8]) -> Result<Option<AutomationRuntime>, String> {
    let wire: RuntimeWire =
        serde_json::from_slice(bytes).map_err(|_| super::runtime_store::runtime_error())?;
    if wire.schema_version != AUTOMATION_RUNTIME_SCHEMA_VERSION || wire.occurrences.len() > 128 {
        return Err(super::runtime_store::runtime_error());
    }
    Ok(Some(AutomationRuntime {
        schema_version: wire.schema_version,
        last_checked_at: wire.last_checked_at,
        occurrences: wire.occurrences.into_iter().map(from_wire).collect(),
    }))
}

fn to_wire(item: &AutomationOccurrence) -> Result<OccurrenceWire, String> {
    let common = CommonWire {
        id: item.id,
        automation_id: item.automation_id,
        scheduled_for: item.scheduled_for,
        created_at: item.created_at,
        updated_at: item.updated_at,
    };
    match item.state {
        OccurrenceState::Pending => Ok(OccurrenceWire::Pending {
            common,
            coalesced_count: item.coalesced_count.ok_or_else(error)?,
            last_scheduled_for: item.last_scheduled_for.ok_or_else(error)?,
        }),
        OccurrenceState::Running => Ok(OccurrenceWire::Running {
            common,
            started_at: item.started_at.ok_or_else(error)?,
        }),
        OccurrenceState::Terminal => Ok(OccurrenceWire::Terminal {
            common,
            started_at: item.started_at,
            result: result_to_wire(item.result.as_ref().ok_or_else(error)?),
        }),
    }
}

fn from_wire(wire: OccurrenceWire) -> AutomationOccurrence {
    match wire {
        OccurrenceWire::Pending {
            common,
            coalesced_count,
            last_scheduled_for,
        } => build(
            common,
            OccurrenceState::Pending,
            Some(coalesced_count),
            Some(last_scheduled_for),
            None,
            None,
        ),
        OccurrenceWire::Running { common, started_at } => build(
            common,
            OccurrenceState::Running,
            None,
            None,
            Some(started_at),
            None,
        ),
        OccurrenceWire::Terminal {
            common,
            started_at,
            result,
        } => build(
            common,
            OccurrenceState::Terminal,
            None,
            None,
            started_at,
            Some(result_from_wire(result)),
        ),
    }
}

fn build(
    common: CommonWire,
    state: OccurrenceState,
    coalesced_count: Option<u32>,
    last_scheduled_for: Option<DateTime<Utc>>,
    started_at: Option<DateTime<Utc>>,
    result: Option<OccurrenceResult>,
) -> AutomationOccurrence {
    AutomationOccurrence {
        id: common.id,
        automation_id: common.automation_id,
        scheduled_for: common.scheduled_for,
        state,
        created_at: common.created_at,
        updated_at: common.updated_at,
        coalesced_count,
        last_scheduled_for,
        started_at,
        result,
    }
}

fn result_to_wire(value: &OccurrenceResult) -> ResultWire {
    ResultWire {
        status: match value.status {
            OccurrenceResultStatus::Ok => ResultStatusWire::Ok,
            OccurrenceResultStatus::Error => ResultStatusWire::Error,
            OccurrenceResultStatus::Missed => ResultStatusWire::Missed,
            OccurrenceResultStatus::Cancelled => ResultStatusWire::Cancelled,
            OccurrenceResultStatus::Interrupted => ResultStatusWire::Interrupted,
        },
        finished_at: value.finished_at,
        error_code: value.error_code.clone(),
        session_id: value.session_id.clone(),
        tokens: value.tokens,
        missed_count: value.missed_count,
        first_scheduled_for: value.first_scheduled_for,
        last_scheduled_for: value.last_scheduled_for,
    }
}

fn result_from_wire(value: ResultWire) -> OccurrenceResult {
    OccurrenceResult {
        status: match value.status {
            ResultStatusWire::Ok => OccurrenceResultStatus::Ok,
            ResultStatusWire::Error => OccurrenceResultStatus::Error,
            ResultStatusWire::Missed => OccurrenceResultStatus::Missed,
            ResultStatusWire::Cancelled => OccurrenceResultStatus::Cancelled,
            ResultStatusWire::Interrupted => OccurrenceResultStatus::Interrupted,
        },
        finished_at: value.finished_at,
        error_code: value.error_code,
        session_id: value.session_id,
        tokens: value.tokens,
        missed_count: value.missed_count,
        first_scheduled_for: value.first_scheduled_for,
        last_scheduled_for: value.last_scheduled_for,
    }
}

fn error() -> String {
    super::runtime_store::runtime_error()
}
