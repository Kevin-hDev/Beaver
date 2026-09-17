use super::{ModelInfo, MODELS};

pub(crate) fn snapshot(provider_id: &str) -> Option<Vec<ModelInfo>> {
    let registry = MODELS.read().ok()?;
    let mut models = registry
        .providers
        .get(provider_id)?
        .values()
        .cloned()
        .collect::<Vec<_>>();
    models.sort_by(|left, right| left.id.cmp(&right.id));
    Some(models)
}
