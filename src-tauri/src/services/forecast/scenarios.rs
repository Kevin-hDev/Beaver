use super::{
    scenario_context::{self, ScenarioCovariateAdjustment},
    scenario_percent, sidecar, storage,
    types::{ForecastResult, Scenario, MAX_SCENARIOS},
};
use serde::{Deserialize, Serialize};

const MAX_SCENARIO_NAME_LEN: usize = 80;
const MAX_SCENARIO_DESCRIPTION_LEN: usize = 500;
const MIN_ADJUSTMENT_PERCENT: f64 = -95.0;
const MAX_ADJUSTMENT_PERCENT: f64 = 500.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioRequest {
    pub analysis_id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_scenario_kind")]
    pub scenario_kind: String,
    #[serde(default)]
    pub adjustment_percent: f64,
    #[serde(default)]
    pub covariate_adjustments: Vec<ScenarioCovariateAdjustment>,
    #[serde(default)]
    pub target_series_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioUpdateRequest {
    pub analysis_id: String,
    pub scenario_id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_scenario_kind")]
    pub scenario_kind: String,
    #[serde(default)]
    pub adjustment_percent: f64,
    #[serde(default)]
    pub covariate_adjustments: Vec<ScenarioCovariateAdjustment>,
    #[serde(default)]
    pub target_series_id: Option<String>,
}

fn default_scenario_kind() -> String {
    "percent_adjustment".into()
}

pub async fn create(
    request: ScenarioRequest,
    chronos: Option<&sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    validate_request(&request)?;
    let mut analysis = storage::load(&request.analysis_id).await?;
    if analysis.scenarios.len() >= MAX_SCENARIOS {
        return Err("Trop de scénarios".into());
    }

    let scenario = build_scenario(
        &analysis,
        uuid::Uuid::new_v4().to_string(),
        &request,
        chronos,
    )
    .await?;
    analysis.scenarios.push(scenario);
    storage::save(&mut analysis).await?;
    Ok(analysis)
}

pub async fn update(
    request: ScenarioUpdateRequest,
    chronos: Option<&sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    validate_request(&ScenarioRequest {
        analysis_id: request.analysis_id.clone(),
        name: request.name.clone(),
        description: request.description.clone(),
        scenario_kind: request.scenario_kind.clone(),
        adjustment_percent: request.adjustment_percent,
        covariate_adjustments: request.covariate_adjustments.clone(),
        target_series_id: request.target_series_id.clone(),
    })?;
    let mut analysis = storage::load(&request.analysis_id).await?;
    let Some(index) = analysis
        .scenarios
        .iter()
        .position(|scenario| scenario.id == request.scenario_id)
    else {
        return Err("Scénario introuvable".into());
    };

    analysis.scenarios[index] = build_scenario(
        &analysis,
        request.scenario_id,
        &ScenarioRequest {
            analysis_id: request.analysis_id,
            name: request.name,
            description: request.description,
            scenario_kind: request.scenario_kind,
            adjustment_percent: request.adjustment_percent,
            covariate_adjustments: request.covariate_adjustments,
            target_series_id: request.target_series_id,
        },
        chronos,
    )
    .await?;
    storage::save(&mut analysis).await?;
    Ok(analysis)
}

pub async fn delete(analysis_id: &str, scenario_id: &str) -> Result<ForecastResult, String> {
    let mut analysis = storage::load(analysis_id).await?;
    let before = analysis.scenarios.len();
    analysis
        .scenarios
        .retain(|scenario| scenario.id != scenario_id);
    if analysis.scenarios.len() == before {
        return Err("Scénario introuvable".into());
    }
    storage::save(&mut analysis).await?;
    Ok(analysis)
}

fn validate_request(request: &ScenarioRequest) -> Result<(), String> {
    let name = request.name.trim();
    if name.is_empty() || name.len() > MAX_SCENARIO_NAME_LEN {
        return Err("Nom de scénario invalide".into());
    }
    if let Some(description) = request.description.as_ref() {
        if description.len() > MAX_SCENARIO_DESCRIPTION_LEN {
            return Err("Description de scénario invalide".into());
        }
    }
    if request.scenario_kind != "percent_adjustment"
        && request.scenario_kind != "context_adjustment"
    {
        return Err("Type de scénario invalide".into());
    }
    if request.scenario_kind == "percent_adjustment"
        && !valid_adjustment(request.adjustment_percent)
    {
        return Err("Ajustement de scénario invalide".into());
    }
    Ok(())
}

fn valid_adjustment(value: f64) -> bool {
    value.is_finite() && (MIN_ADJUSTMENT_PERCENT..=MAX_ADJUSTMENT_PERCENT).contains(&value)
}

fn clean_description(description: Option<String>) -> Option<String> {
    description
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn build_scenario(
    analysis: &ForecastResult,
    id: String,
    request: &ScenarioRequest,
    chronos: Option<&sidecar::ChronosSidecar>,
) -> Result<Scenario, String> {
    if request.scenario_kind == "context_adjustment" {
        return scenario_context::build(
            analysis,
            id,
            request.name.trim().to_string(),
            clean_description(request.description.clone()),
            request.covariate_adjustments.clone(),
            request.target_series_id.clone(),
            chronos,
        )
        .await;
    }

    Ok(scenario_percent::build_percent_scenario(
        analysis,
        id,
        request.name.trim().to_string(),
        clean_description(request.description.clone()),
        request.adjustment_percent,
    ))
}
