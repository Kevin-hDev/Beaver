use crate::services::agent_local::modelfile_parser::parse_modelfile;
use crate::services::agent_local::ollama_client::OllamaClient;
use crate::services::agent_local::system_prompt_types::PromptTier;

pub struct ContextWindows {
    pub native: u64,
    pub configured: u64,
    pub prompt_tier: Option<PromptTier>,
}

pub async fn resolve_ollama(model: &str) -> ContextWindows {
    let Ok(ollama) = OllamaClient::from_global() else {
        return unavailable_ollama_context(model);
    };
    resolve_ollama_with_client(&ollama, model).await
}

pub async fn resolve_ollama_with_client(ollama: &OllamaClient, model: &str) -> ContextWindows {
    let loaded = ollama.loaded_context_length(model).await.ok().flatten();
    let info = fetch_ollama_model_info(ollama, model).await;
    let native = info.context_length;
    let configured = select_ollama_context(
        loaded,
        info.num_ctx_from_modelfile,
        native,
        u64::from(crate::services::gpu_detect::compute_default_num_ctx()),
    );
    ContextWindows {
        native,
        configured,
        prompt_tier: Some(info.prompt_tier),
    }
}

pub async fn resolve_api(provider: &str, model: &str) -> ContextWindows {
    let native = lookup_api_context(provider, model).await;
    ContextWindows {
        native,
        configured: native,
        prompt_tier: None,
    }
}

pub async fn resolve(provider: &str, model: &str) -> ContextWindows {
    if crate::services::llm::route_profile::is_local(provider) {
        resolve_ollama(model).await
    } else {
        resolve_api(provider, model).await
    }
}

struct OllamaModelContext {
    context_length: u64,
    num_ctx_from_modelfile: Option<u64>,
    prompt_tier: PromptTier,
}

async fn fetch_ollama_model_info(ollama: &OllamaClient, model: &str) -> OllamaModelContext {
    let Ok(info) = ollama.show_model(model).await else {
        return unavailable_ollama_info(model);
    };
    let parsed = parse_modelfile(&info.modelfile);
    OllamaModelContext {
        context_length: info.context_length,
        num_ctx_from_modelfile: parsed.parameters.get("num_ctx").and_then(|v| v.as_u64()),
        prompt_tier: crate::services::agent_local::model_size::detect_ollama_tier(
            &info.parameter_size,
            model,
        ),
    }
}

#[cfg(test)]
fn ollama_model_context_from_json(model: &str, json: &serde_json::Value) -> OllamaModelContext {
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
        prompt_tier: crate::services::agent_local::model_size::detect_ollama_tier(
            json["details"]["parameter_size"]
                .as_str()
                .unwrap_or_default(),
            model,
        ),
    }
}

pub(crate) fn select_ollama_context(
    loaded: Option<u64>,
    configured: Option<u64>,
    native: u64,
    hardware_default: u64,
) -> u64 {
    if let Some(value) = loaded.filter(|value| *value > 0) {
        return value;
    }
    if let Some(value) = configured.filter(|value| *value > 0) {
        return value;
    }
    match (native, hardware_default) {
        (native, hardware) if native > 0 && hardware > 0 => native.min(hardware),
        (native, _) if native > 0 => native,
        (_, hardware) => hardware,
    }
}

fn unavailable_ollama_context(model: &str) -> ContextWindows {
    let info = unavailable_ollama_info(model);
    ContextWindows {
        native: 0,
        configured: 0,
        prompt_tier: Some(info.prompt_tier),
    }
}

fn unavailable_ollama_info(model: &str) -> OllamaModelContext {
    OllamaModelContext {
        context_length: 0,
        num_ctx_from_modelfile: None,
        prompt_tier: crate::services::agent_local::model_size::detect_tier(model),
    }
}

async fn lookup_api_context(provider: &str, model: &str) -> u64 {
    crate::services::llm::model_context_length(provider, model)
        .await
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
            prompt_tier: None,
        };
        assert_eq!(ctx.native, 131_072);
        assert_eq!(ctx.configured, 32_768);
    }

    #[test]
    fn next_ollama_request_uses_loaded_then_modelfile_then_safe_native_limit() {
        assert_eq!(
            select_ollama_context(Some(65_536), Some(32_768), 16_384, 8_192),
            65_536
        );
        assert_eq!(
            select_ollama_context(Some(0), Some(32_768), 16_384, 8_192),
            32_768
        );
        assert_eq!(select_ollama_context(None, None, 16_384, 8_192), 8_192);
        assert_eq!(select_ollama_context(None, Some(0), 16_384, 8_192), 8_192);
        assert_eq!(select_ollama_context(None, None, 4_096, 8_192), 4_096);
        assert_eq!(select_ollama_context(None, None, 0, 0), 0);
    }

    #[test]
    fn ollama_context_uses_parameter_size_for_the_runtime_prompt_tier() {
        let context = ollama_model_context_from_json(
            "misleading:70b",
            &serde_json::json!({
                "details": { "parameter_size": "7B" },
                "model_info": {
                    "general.architecture": "gemma",
                    "gemma.context_length": 8192
                }
            }),
        );

        assert_eq!(context.prompt_tier, PromptTier::Compact);
        assert_eq!(context.context_length, 8192);
    }

    #[tokio::test]
    async fn provider_registry_supplies_verified_api_contexts() {
        assert_eq!(lookup_api_context("openai", "gpt-5.6-sol").await, 1_050_000);
        assert_eq!(
            lookup_api_context("openrouter", "openai/gpt-5.6-terra").await,
            1_050_000
        );
        assert_eq!(lookup_api_context("xai", "grok-4.5").await, 500_000);
        assert_eq!(
            lookup_api_context("mistral", "mistral-small-latest").await,
            262_144
        );
        assert_eq!(
            lookup_api_context("moonshot-oauth", "k3-256k").await,
            262_144
        );
        assert_eq!(
            lookup_api_context("codex-oauth", "gpt-5.6-luna").await,
            258_400
        );
    }
}
