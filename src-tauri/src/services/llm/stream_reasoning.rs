use serde_json::Value;

pub fn apply(
    payload: &mut Value,
    provider_id: &str,
    model: &str,
    think: bool,
    reasoning_mode: Option<&str>,
) {
    if reasoning_mode.is_none() && !think {
        return;
    }
    match provider_id {
        "zai" => apply_zai(payload, model, reasoning_mode),
        "openrouter" => apply_openrouter(payload, model, think, reasoning_mode),
        "deepseek" => apply_deepseek(payload, reasoning_mode),
        "groq" => apply_groq(payload, model, think, reasoning_mode),
        "mistral" => apply_mistral(payload, think, reasoning_mode),
        "moonshot" => apply_moonshot(payload, model, think, reasoning_mode),
        "google" => apply_google(payload, model, think, reasoning_mode),
        "openai" => apply_openai(payload, model, think, reasoning_mode),
        "xai" => apply_xai(payload, model, reasoning_mode),
        _ => {}
    }
}

fn apply_thinking(payload: &mut Value, reasoning_mode: Option<&str>) {
    payload["thinking"] = serde_json::json!({
        "type": if reasoning_mode == Some("off") { "disabled" } else { "enabled" }
    });
}

fn apply_zai(payload: &mut Value, model: &str, reasoning_mode: Option<&str>) {
    apply_thinking(payload, reasoning_mode);
    if model.to_lowercase().starts_with("glm-5.2") {
        if let Some(effort) = crate::services::reasoning::zai_effort(reasoning_mode) {
            payload["reasoning_effort"] = effort.into();
        }
    }
}

fn apply_openrouter(payload: &mut Value, model: &str, think: bool, reasoning_mode: Option<&str>) {
    let supported = crate::services::reasoning::supported_modes("openrouter", model, true);
    if reasoning_mode.is_some_and(|mode| !supported.contains(&mode)) {
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
        Some("xhigh") => "max",
        _ => "high",
    }
    .into();
}

fn apply_groq(payload: &mut Value, model: &str, think: bool, reasoning_mode: Option<&str>) {
    let model = model.to_lowercase();
    if crate::services::llm::providers::groq::is_qwen_switchable(&model) {
        payload["reasoning_effort"] = if reasoning_mode == Some("off") {
            "none"
        } else {
            "default"
        }
        .into();
        payload["reasoning_format"] = "parsed".into();
    } else if crate::services::llm::providers::groq::is_gpt_oss_effort(&model) && think {
        if let Some(effort) = crate::services::reasoning::simple_effort(reasoning_mode) {
            payload["reasoning_effort"] = effort.into();
            payload["include_reasoning"] = true.into();
        }
    } else if think {
        payload["include_reasoning"] = true.into();
    }
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
    if crate::services::llm::providers::moonshot::is_forced_thinking(&model) {
        let mut thinking = serde_json::json!({ "type": "enabled" });
        if crate::services::llm::providers::moonshot::is_k3(&model) {
            let effort = reasoning_mode
                .filter(|effort| matches!(*effort, "low" | "high" | "max"))
                .unwrap_or("max");
            thinking["effort"] = effort.into();
        }
        payload["thinking"] = thinking;
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

fn apply_openai(payload: &mut Value, model: &str, think: bool, reasoning_mode: Option<&str>) {
    if !think && reasoning_mode != Some("off") {
        return;
    }
    let supported = crate::services::reasoning::supported_modes("openai", model, true);
    if reasoning_mode.is_some_and(|mode| !supported.contains(&mode)) {
        return;
    }
    if let Some(effort) = crate::services::reasoning::openai_effort(reasoning_mode) {
        payload["reasoning_effort"] = effort.into();
    }
}
