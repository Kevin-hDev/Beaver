use super::types::ModelInfo;

pub(super) fn supports_parameter(model: &ModelInfo, parameter: &str) -> bool {
    model
        .supported_parameters
        .as_ref()
        .is_some_and(|parameters| parameters.iter().any(|candidate| candidate == parameter))
}

pub(super) fn output_limit_field(model: &ModelInfo) -> &'static str {
    if supports_parameter(model, "max_completion_tokens") {
        "max_completion_tokens"
    } else {
        "max_tokens"
    }
}
