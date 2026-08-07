use crate::services::agent_local::model_customizations;
use crate::services::agent_local::modelfile_parser::{
    merge_parameter, parse_modelfile, parse_param_value,
};
use crate::services::agent_local::ollama_base_url;
use crate::services::agent_local::ollama_model_helpers::{
    build_model_from_tags, dedupe_by_digest, parse_show_response,
};
use crate::services::agent_local::types_ollama::{ModelInfo, OllamaModel};
use reqwest::Client;
use std::time::Duration;
const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RUNTIME_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

pub struct OllamaClient {
    client: Client,
    base_url: Option<String>,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self::build(None)
    }

    fn build(base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self { client, base_url }
    }

    fn base_url(&self) -> String {
        self.base_url.clone().unwrap_or_else(ollama_base_url)
    }

    #[cfg(test)]
    pub(crate) fn with_base_url(base_url: &str) -> Result<Self, String> {
        let parsed = url::Url::parse(base_url).map_err(|_| "invalid Ollama test URL")?;
        let loopback = parsed.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|address| address.is_loopback())
        });
        if parsed.scheme() != "http"
            || !loopback
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err("invalid Ollama test URL".to_string());
        }
        Ok(Self::build(Some(
            base_url.trim_end_matches('/').to_string(),
        )))
    }

    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url()))
            .timeout(TIMEOUT)
            .send()
            .await
            .is_ok()
    }

    pub async fn loaded_context_length(&self, name: &str) -> Result<Option<u64>, String> {
        model_customizations::validate_model_name(name)?;
        let resp = self
            .client
            .get(format!("{}/api/ps", self.base_url()))
            .timeout(TIMEOUT)
            .send()
            .await
            .map_err(|error| {
                ::log::warn!("[ollama] /api/ps: {error}");
                "ollama-runtime-error".to_string()
            })?;
        if !resp.status().is_success() {
            return Err("ollama-runtime-error".into());
        }
        let body = resp
            .bytes()
            .await
            .map_err(|_| "ollama-runtime-error".to_string())?;
        if body.len() > MAX_RUNTIME_RESPONSE_BYTES {
            return Err("ollama-response-too-large".into());
        }
        super::ollama_runtime::loaded_context_length(&body, name)
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        let resp = self
            .client
            .get(format!("{}/api/tags", self.base_url()))
            .send()
            .await
            .map_err(|e| {
                ::log::warn!("[ollama] /api/tags: {e}");
                "ollama-connection-error".to_string()
            })?;
        let body = resp.bytes().await.map_err(|e| e.to_string())?;
        if body.len() > 10 * 1024 * 1024 {
            return Err("ollama-response-too-large".into());
        }
        let json: serde_json::Value = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
        let models = json["models"].as_array().ok_or("ollama-invalid-response")?;

        let mut raw = Vec::new();
        for m in models.iter().take(500) {
            let name = m["name"].as_str().unwrap_or_default().to_string();
            let is_customized = model_customizations::is_model_customized(&name);
            let info = self.show_model(&name).await.ok();
            raw.push(build_model_from_tags(m, info, is_customized));
        }
        Ok(dedupe_by_digest(raw))
    }

    pub async fn get_modelfile(&self, name: &str) -> Result<String, String> {
        let info = self.show_model(name).await?;
        Ok(info.modelfile)
    }

    pub async fn update_modelfile(&self, name: &str, content: &str) -> Result<(), String> {
        super::ollama_modelfile_create::create_from_modelfile(name, content).await
    }

    pub async fn update_parameters(
        &self,
        name: &str,
        entries: Vec<(String, String)>,
    ) -> Result<(), String> {
        super::ollama_parameter_validation::validate_parameter_entries(&entries)?;
        let current = self.get_modelfile(name).await?;
        let mut parsed = parse_modelfile(&current);
        parsed.parameters.clear();
        for (k, v) in entries {
            let key = k.trim();
            let raw = v.trim();
            if key.is_empty() || raw.is_empty() {
                continue;
            }
            let value = parse_param_value(raw);
            merge_parameter(&mut parsed.parameters, key, value);
        }
        parsed.from = Some(name.to_string());
        parsed.license = None;
        let payload = parsed.to_api_payload(name);
        self.post_create(&payload).await
    }

    pub(crate) async fn post_create(&self, payload: &serde_json::Value) -> Result<(), String> {
        let enriched = super::ollama_create_payload::non_streaming(payload)?;
        let resp = self
            .client
            .post(format!("{}/api/create", self.base_url()))
            .json(&enriched)
            .send()
            .await
            .map_err(|e| {
                ::log::error!("[ollama] /api/create send: {e}");
                "ollama-create-error".to_string()
            })?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            ::log::error!(
                "[ollama] /api/create failed ({status}): {}",
                crate::services::llm::sanitize_log_body(&body)
            );
            return Err("ollama-create-error".to_string());
        }
        Ok(())
    }

    pub async fn show_model(&self, name: &str) -> Result<ModelInfo, String> {
        let resp = self
            .client
            .post(format!("{}/api/show", self.base_url()))
            .json(&serde_json::json!({ "model": name }))
            .send()
            .await
            .map_err(|e| {
                ::log::warn!("[ollama] /api/show: {e}");
                "ollama-show-error".to_string()
            })?;
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(parse_show_response(name, &json))
    }
}
