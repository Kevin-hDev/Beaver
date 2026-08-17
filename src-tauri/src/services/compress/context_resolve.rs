use crate::services::agent_local::modelfile_parser::parse_modelfile;
use crate::services::agent_local::ollama_client::OllamaClient;

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
    let Ok(ollama) = OllamaClient::from_global() else {
        return OllamaModelContext {
            context_length: 0,
            num_ctx_from_modelfile: None,
        };
    };
    let Ok(base_url) = ollama.base_url().await else {
        return OllamaModelContext {
            context_length: 0,
            num_ctx_from_modelfile: None,
        };
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/api/show"))
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
        return crate::services::codex_client::model_catalog::context_length(model).await;
    }

    let provider = crate::services::llm::route::canonical_provider_id(provider);
    if let Some(context) =
        crate::services::llm::provider_model_lookup::local_limits(provider, model)
            .and_then(|limits| limits.context_window)
    {
        return context as u64;
    }
    if let Some(context) = crate::services::llm::runtime_models::lookup(provider, model)
        .and_then(|model| model.context_length)
    {
        return context as u64;
    }
    crate::services::llm::provider_model_lookup::limits(provider, model)
        .await
        .and_then(|limits| limits.context_window)
        .unwrap_or(0) as u64
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
