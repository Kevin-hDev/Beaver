use super::catalog::{ForecastModelSpec, ForecastProviderSpec};
use super::{model_details_github, model_details_huggingface};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ForecastModelDetails {
    pub description_short: String,
    pub description_long_markdown: String,
    pub source_url: String,
    pub source_label: String,
    pub license: Option<String>,
    pub pipeline_tag: Option<String>,
    pub library_name: Option<String>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    pub tags: Vec<String>,
}

pub async fn fetch(
    model: &ForecastModelSpec,
    provider: Option<&ForecastProviderSpec>,
) -> Result<ForecastModelDetails, String> {
    if let Some(repo) = model.github_repo {
        return model_details_github::fetch_github(repo, model.github_revision).await;
    }
    if let Some(repo) = model.hf_repo {
        return model_details_huggingface::fetch(repo, model.hf_revision).await;
    }
    Ok(provider_fallback(model, provider))
}

fn provider_fallback(
    model: &ForecastModelSpec,
    provider: Option<&ForecastProviderSpec>,
) -> ForecastModelDetails {
    ForecastModelDetails {
        description_short: provider
            .map(|p| p.short_description.to_string())
            .unwrap_or_else(|| model.display_name.to_string()),
        description_long_markdown: String::new(),
        source_url: provider
            .map(|p| p.signup_url.to_string())
            .unwrap_or_default(),
        source_label: provider
            .map(|p| p.display_name.to_string())
            .unwrap_or_else(|| model.provider_id.to_string()),
        license: None,
        pipeline_tag: None,
        library_name: None,
        downloads: None,
        likes: None,
        tags: Vec::new(),
    }
}
