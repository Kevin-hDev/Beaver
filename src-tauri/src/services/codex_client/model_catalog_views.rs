use super::{load_catalog, unavailable, CACHE};
use crate::services::llm::types::ModelInfo;

pub async fn available_models() -> Result<Vec<ModelInfo>, String> {
    let models = load_catalog().await?;
    let visible = models
        .into_iter()
        .filter(|model| model.visible)
        .map(|model| model.info)
        .collect::<Vec<_>>();
    if visible.is_empty() {
        Err(unavailable())
    } else {
        Ok(visible)
    }
}

pub async fn cached_models() -> Option<Vec<ModelInfo>> {
    CACHE.lock().await.catalog.as_ref().map(|catalog| {
        catalog
            .models
            .iter()
            .filter(|model| model.visible)
            .map(|model| model.info.clone())
            .collect()
    })
}
