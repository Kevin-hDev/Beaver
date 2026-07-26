use crate::services::agent_local::modelfile_parser::parse_modelfile;
use crate::services::agent_local::ollama_base_url;

pub struct ContextWindows {
    pub native: u64,
    pub configured: u64,
}

pub async fn resolve_ollama(model: &str) -> ContextWindows {
    let info = fetch_ollama_model_info(model).await;
    let native = info.context_length;
    let configured = info.num_ctx_from_modelfile.unwrap_or_else(|| {
        let hardware_default = crate::services::gpu_detect::compute_default_num_ctx() as u64;
        if hardware_default > 0 && native > 0 {
            hardware_default.min(native)
        } else {
            native
        }
    });
    ContextWindows { native, configured }
}

pub async fn resolve_api(provider: &str, model: &str) -> ContextWindows {
    let native = lookup_api_context(provider, model).await;
    ContextWindows {
        native,
        configured: native,
    }
}

struct OllamaModelContext {
    context_length: u64,
    num_ctx_from_modelfile: Option<u64>,
}

async fn fetch_ollama_model_info(model: &str) -> OllamaModelContext {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/show", ollama_base_url()))
        .json(&serde_json::json!({ "model": model }))
        .send()
        .await;

    let json = match resp {
        Ok(r) => r.json::<serde_json::Value>().await.unwrap_or_default(),
        Err(_) => {
            return OllamaModelContext {
                context_length: 0,
                num_ctx_from_modelfile: None,
            }
        }
    };

    let mi = &json["model_info"];
    let arch = mi["general.architecture"].as_str().unwrap_or("");
    let context_length = mi[format!("{arch}.context_length")].as_u64().unwrap_or(0);

    let num_ctx = json
        .get("modelfile")
        .and_then(|v| v.as_str())
        .and_then(|mf| {
            let parsed = parse_modelfile(mf);
            parsed.parameters.get("num_ctx").and_then(|v| v.as_u64())
        });

    OllamaModelContext {
        context_length,
        num_ctx_from_modelfile: num_ctx,
    }
}

async fn lookup_api_context(provider: &str, model: &str) -> u64 {
    if provider == "codex-oauth" {
        return crate::services::codex_client::types::CODEX_MODELS
            .iter()
            .find(|spec| spec.id == model)
            .map(|spec| spec.context_length as u64)
            .unwrap_or(128_000);
    }

    if matches!(provider, "openai" | "openrouter") {
        if let Some(context) = crate::services::llm::providers::openai::context_length(model) {
            return context as u64;
        }
    }
    if provider == "xai" {
        if let Some(context) = crate::services::llm::providers::xai::context_length(model) {
            return context as u64;
        }
    }
    if let Some(model) = crate::services::llm::runtime_models::lookup(provider, model) {
        return model.context_length.unwrap_or(0) as u64;
    }
    use crate::services::llm::model_registry;

    let reg = model_registry::get_lock().read().await;
    let prefix = match provider {
        "google" => "gemini",
        other => other,
    };

    let key = format!("{prefix}/{model}");
    let entry = reg.get(&key).or_else(|| reg.get(model)).or_else(|| {
        let stripped = model.rsplit_once('/').map(|(_, n)| n).unwrap_or(model);
        let key2 = format!("{prefix}/{stripped}");
        reg.get(&key2).or_else(|| reg.get(stripped))
    });

    entry
        .and_then(|e| e.max_input_tokens.or(e.max_tokens))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_context_windows() {
        let ctx = ContextWindows {
            native: 131_072,
            configured: 32_768,
        };
        assert_eq!(ctx.native, 131_072);
        assert_eq!(ctx.configured, 32_768);
    }

    #[tokio::test]
    async fn official_api_contexts_override_registry() {
        assert_eq!(lookup_api_context("openai", "gpt-5.6-sol").await, 1_050_000);
        assert_eq!(
            lookup_api_context("openrouter", "openai/gpt-5.6-terra-pro").await,
            1_050_000
        );
        assert_eq!(lookup_api_context("xai", "grok-4.5").await, 500_000);
        assert_eq!(
            lookup_api_context("codex-oauth", "gpt-5.6-luna").await,
            372_000
        );
    }
}
