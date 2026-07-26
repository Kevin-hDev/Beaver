use serde::Serialize;
use serde_json::Value;

use super::data_quality::DataProfile;
use super::evaluation::types::{BacktestIndexSummary, BacktestMetrics};
use super::hardware_profile::{HardwareProfile, ResourceFit};
use super::limits;

#[path = "auto_selection_candidate.rs"]
mod candidate;
#[path = "auto_selection_rank.rs"]
mod rank;

#[derive(Debug, Clone, Serialize)]
pub struct AutoCandidate {
    pub model_id: String,
    pub compatibility: &'static str,
    pub resource_fit: ResourceFit,
    pub reasons: Vec<&'static str>,
    pub interval_capability: super::interval_capability::IntervalCapability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtest: Option<CandidateBacktest>,
    #[serde(skip)]
    pub evidence: Option<BacktestIndexSummary>,
    #[serde(skip)]
    pub(crate) estimated_ram_mb: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateBacktest {
    pub evaluated_at: String,
    pub windows: usize,
    pub metrics: BacktestMetrics,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    pub beats_best_baseline: Option<bool>,
    pub calibration: Option<super::evaluation::types::IntervalCalibration>,
}

pub struct AutoSelection {
    pub candidates: Vec<AutoCandidate>,
    pub basis: &'static str,
    pub requested_model: Option<RequestedModelStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestedModelStatus {
    pub model_id: String,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusion_reason: Option<&'static str>,
    pub runtime_setup_required: bool,
}

#[cfg(test)]
pub fn select(
    models: &[Value],
    profile: &DataProfile,
    cloud_allowed: bool,
    hardware: HardwareProfile,
    evidence: &[BacktestIndexSummary],
) -> AutoSelection {
    select_with_requested_model(models, profile, cloud_allowed, hardware, evidence, None)
}

pub fn select_with_requested_model(
    models: &[Value],
    profile: &DataProfile,
    cloud_allowed: bool,
    hardware: HardwareProfile,
    evidence: &[BacktestIndexSummary],
    requested_model_id: Option<&str>,
) -> AutoSelection {
    let mut requested_model = requested_model_id.map(|id| RequestedModelStatus {
        model_id: id.to_string(),
        status: "excluded",
        exclusion_reason: Some("unknown_model"),
        runtime_setup_required: false,
    });
    let mut candidates = Vec::new();
    for model in models {
        let is_requested = model["id"].as_str() == requested_model_id;
        match candidate::evaluate(
            model,
            profile,
            cloud_allowed,
            hardware,
            evidence,
            is_requested,
        ) {
            Ok(candidate) => {
                if is_requested {
                    requested_model = Some(RequestedModelStatus {
                        model_id: candidate.model_id.clone(),
                        status: "candidate",
                        exclusion_reason: None,
                        runtime_setup_required: candidate
                            .reasons
                            .contains(&"runtime_setup_required"),
                    });
                }
                candidates.push(candidate);
            }
            Err(reason) if is_requested => {
                if let Some(status) = requested_model.as_mut() {
                    status.exclusion_reason = Some(reason);
                }
            }
            Err(_) => {}
        }
    }
    let rolling = candidates
        .iter()
        .any(|candidate| candidate.backtest.is_some());
    candidates.sort_by(|left, right| rank::compare(left, right, rolling));
    if let Some(first) = candidates.first_mut().filter(|candidate| {
        !rolling
            || candidate
                .backtest
                .as_ref()
                .and_then(|backtest| backtest.beats_best_baseline)
                == Some(true)
    }) {
        first.compatibility = "recommended";
    }
    promote_requested(&mut candidates, requested_model_id);
    AutoSelection {
        candidates,
        basis: if rolling {
            "rolling_backtest"
        } else {
            "capabilities_and_resources"
        },
        requested_model,
    }
}

fn promote_requested(candidates: &mut Vec<AutoCandidate>, requested_model_id: Option<&str>) {
    let requested = requested_model_id.and_then(|id| {
        candidates
            .iter()
            .position(|candidate| candidate.model_id == id)
            .map(|position| candidates.remove(position))
    });
    if let Some(mut requested) = requested {
        requested.compatibility = "requested";
        candidates.truncate(limits::MAX_AUTO_CANDIDATES.saturating_sub(1));
        candidates.insert(0, requested);
    } else {
        candidates.truncate(limits::MAX_AUTO_CANDIDATES);
    }
}
