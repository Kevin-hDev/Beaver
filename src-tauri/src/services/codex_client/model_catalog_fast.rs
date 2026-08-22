use super::model_catalog_wire::WireModel;

pub(super) fn supports_fast_mode(model: &WireModel) -> bool {
    model
        .service_tiers
        .0
        .iter()
        .any(|tier| valid_tier_id(&tier.id) && tier.id == "priority")
}

fn valid_tier_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || matches!(byte, b'-' | b'_'))
}
