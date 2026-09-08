use serde_json::Value;

pub(super) fn apply(payload: &mut Value, model: &str, reasoning_mode: Option<&str>) {
    if model.eq_ignore_ascii_case("glm-5.3") {
        // GLM 5.3 raisonne toujours : "off" est donc replié sur le défaut officiel.
        super::stream_reasoning::apply_thinking(payload, Some("max"));
        let contract = crate::services::llm::provider_model_lookup::local_reasoning("zai", model);
        payload["reasoning_effort"] = resolve_glm_53_effort(reasoning_mode, contract).into();
        return;
    }
    if model.eq_ignore_ascii_case("glm-5.3-flash") {
        payload["thinking"] = serde_json::json!({
            "type": "enabled",
            "clear_thinking": false,
        });
        let contract = crate::services::llm::provider_model_lookup::local_reasoning("zai", model);
        payload["reasoning_effort"] = resolve_glm_53_effort(reasoning_mode, contract).into();
        return;
    }
    super::stream_reasoning::apply_thinking(payload, reasoning_mode);
    if model.to_lowercase().starts_with("glm-5.2") {
        if let Some(effort) = crate::services::reasoning::zai_effort(reasoning_mode) {
            payload["reasoning_effort"] = effort.into();
        }
    }
}

pub(super) fn resolve_glm_53_effort(
    reasoning_mode: Option<&str>,
    contract: Option<crate::services::llm::provider_model_lookup::ModelReasoning>,
) -> String {
    contract
        .and_then(|contract| {
            reasoning_mode
                .filter(|mode| contract.modes.iter().any(|candidate| candidate == mode))
                .map(str::to_string)
                .or(contract.default_mode)
        })
        // Dernier filet : GLM 5.3 doit toujours recevoir un effort, même sans registre lisible.
        .unwrap_or_else(|| "max".to_string())
}
