use serde_json::Value;

pub fn apply(
    payload: &mut Value,
    policy: super::route_profile::ParameterPolicy,
    model: &str,
    think: bool,
    reasoning_mode: Option<&str>,
) {
    if reasoning_mode.is_none() && !think {
        return;
    }
    use super::route_profile::ParameterPolicy;
    match policy {
        ParameterPolicy::Zai => apply_zai(payload, model, reasoning_mode),
        ParameterPolicy::DeepSeek => apply_deepseek(payload, reasoning_mode),
        ParameterPolicy::Mistral => apply_mistral(payload, think, reasoning_mode),
        ParameterPolicy::Cerebras => apply_cerebras(payload, think, reasoning_mode),
        ParameterPolicy::OpenRouter => apply_openrouter(payload, model, think, reasoning_mode),
        ParameterPolicy::Moonshot => apply_moonshot(payload, model, think, reasoning_mode),
        ParameterPolicy::Google => apply_google(payload, model, think, reasoning_mode),
        ParameterPolicy::Xai => apply_xai(payload, model, reasoning_mode),
        ParameterPolicy::Qwen => apply_qwen(payload, reasoning_mode),
        ParameterPolicy::Default
        | ParameterPolicy::Responses
        | ParameterPolicy::Ollama
        | ParameterPolicy::Anthropic => {}
    }
}

fn apply_qwen(payload: &mut Value, reasoning_mode: Option<&str>) {
    let off = reasoning_mode == Some("off");
    payload["enable_thinking"] = (!off).into();
    payload["preserve_thinking"] = (!off).into();
    if !off {
        let effort = reasoning_mode
            .filter(|mode| matches!(*mode, "low" | "medium" | "xhigh"))
            .unwrap_or("xhigh");
        payload["reasoning_effort"] = effort.into();
    }
}

fn apply_cerebras(payload: &mut Value, think: bool, reasoning_mode: Option<&str>) {
    if reasoning_mode == Some("off") {
        payload["reasoning_effort"] = "none".into();
        return;
    }
    if !think {
        return;
    }
    if let Some(effort) = crate::services::reasoning::simple_effort(reasoning_mode) {
        payload["reasoning_effort"] = effort.into();
    }
}

fn apply_thinking(payload: &mut Value, reasoning_mode: Option<&str>) {
    payload["thinking"] = serde_json::json!({
        "type": if reasoning_mode == Some("off") { "disabled" } else { "enabled" }
    });
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

fn apply_zai(payload: &mut Value, model: &str, reasoning_mode: Option<&str>) {
    if model.eq_ignore_ascii_case("glm-5.3") {
        // GLM 5.3 raisonne toujours : "off" est donc replié sur le défaut officiel.
        apply_thinking(payload, Some("max"));
        let contract = crate::services::llm::provider_model_lookup::local_reasoning("zai", model);
        payload["reasoning_effort"] = resolve_glm_53_effort(reasoning_mode, contract).into();
        return;
    }
    apply_thinking(payload, reasoning_mode);
    if model.to_lowercase().starts_with("glm-5.2") {
        if let Some(effort) = crate::services::reasoning::zai_effort(reasoning_mode) {
            payload["reasoning_effort"] = effort.into();
        }
    }
}

fn apply_openrouter(payload: &mut Value, model: &str, think: bool, reasoning_mode: Option<&str>) {
    let supported = crate::services::reasoning::supported_modes("openrouter", model, true);
    if reasoning_mode.is_some_and(|mode| !supported.iter().any(|candidate| candidate == mode)) {
        return;
    }
    if reasoning_mode == Some("off") {
        payload["reasoning"] = serde_json::json!({ "effort": "none" });
    } else if think && reasoning_mode == Some("auto") {
        payload["reasoning"] = serde_json::json!({ "enabled": true });
    } else if think {
        if let Some(effort) = crate::services::reasoning::openrouter_effort(reasoning_mode) {
            payload["reasoning"] = serde_json::json!({ "effort": effort });
        }
    }
}

fn apply_deepseek(payload: &mut Value, reasoning_mode: Option<&str>) {
    if reasoning_mode == Some("off") {
        payload["thinking"] = serde_json::json!({ "type": "disabled" });
        return;
    }
    payload["thinking"] = serde_json::json!({ "type": "enabled" });
    payload["reasoning_effort"] = match reasoning_mode {
        Some("low") => "low",
        Some("max") => "max",
        // Compatibilité des sessions créées avant l'exposition du mode `max`.
        Some("xhigh") => "max",
        _ => "high",
    }
    .into();
}

fn apply_mistral(payload: &mut Value, think: bool, reasoning_mode: Option<&str>) {
    if !think && reasoning_mode != Some("off") {
        return;
    }
    if reasoning_mode == Some("off") {
        payload["reasoning_effort"] = "none".into();
    } else if reasoning_mode == Some("high") {
        payload["reasoning_effort"] = "high".into();
    }
}

fn apply_moonshot(payload: &mut Value, model: &str, think: bool, reasoning_mode: Option<&str>) {
    let model = model.to_lowercase();
    if crate::services::llm::providers::moonshot::is_k3(&model) {
        let effort = reasoning_mode
            .filter(|effort| matches!(*effort, "low" | "high" | "max"))
            .unwrap_or("max");
        payload["reasoning_effort"] = effort.into();
        return;
    }
    if crate::services::llm::providers::moonshot::is_forced_thinking(&model) {
        return;
    }
    if reasoning_mode == Some("off") {
        payload["thinking"] = serde_json::json!({ "type": "disabled" });
    } else if think {
        payload["thinking"] = serde_json::json!({ "type": "enabled" });
    }
}

fn apply_xai(payload: &mut Value, model: &str, reasoning_mode: Option<&str>) {
    if let Some(effort) =
        crate::services::llm::providers::xai::reasoning_effort(model, reasoning_mode)
    {
        payload["reasoning_effort"] = effort.into();
    }
}

fn apply_google(payload: &mut Value, model: &str, think: bool, reasoning_mode: Option<&str>) {
    if reasoning_mode == Some("off") {
        payload["reasoning_effort"] = "none".into();
        return;
    }
    if !think {
        return;
    }
    let effort = crate::services::reasoning::simple_effort(reasoning_mode).unwrap_or("medium");
    let mut thinking_config = serde_json::json!({ "include_thoughts": true });
    if is_gemini_25(model) {
        thinking_config["thinking_budget"] = google_thinking_budget(effort).into();
    } else {
        thinking_config["thinking_level"] = effort.into();
    }
    payload["extra_body"]["google"]["thinking_config"] = thinking_config;
}

fn is_gemini_25(model: &str) -> bool {
    model.to_lowercase().contains("gemini-2.5")
}

fn google_thinking_budget(effort: &str) -> u32 {
    match effort {
        "low" => 1_024,
        "high" => 24_576,
        _ => 8_192,
    }
}
