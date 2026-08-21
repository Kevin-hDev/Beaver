use crate::services::agent_local::model_customizations;
use crate::services::agent_local::ollama_model_helpers::{
    build_model_from_tags, dedupe_by_digest, parse_show_response,
};
use crate::services::agent_local::types_ollama::{
    ModelInfo, OllamaModel, OllamaModelEditorData, OllamaParameter,
};
use crate::services::ollama_manager::OllamaManager;
use reqwest::Client;
use std::time::Duration;
use tauri::Manager;
const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RUNTIME_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct OllamaClient {
    client: Client,
    manager: OllamaManager,
    base_url: Option<String>,
}

impl OllamaClient {
    pub fn new(manager: OllamaManager) -> Self {
        Self::build(manager, None)
    }

    pub fn from_global() -> Result<Self, String> {
        let app = super::app_handle_global::get()
            .ok_or_else(|| "ollama-manager-unavailable".to_string())?;
        app.try_state::<Self>()
            .map(|state| state.inner().clone())
            .ok_or_else(|| "ollama-manager-unavailable".to_string())
    }

    fn build(manager: OllamaManager, base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            manager,
            base_url,
        }
    }

    pub(crate) async fn base_url(&self) -> Result<String, String> {
        match &self.base_url {
            Some(url) => Ok(url.clone()),
            None => self
                .manager
                .usable_endpoint()
                .await
                .map(|endpoint| endpoint.as_http_url())
                .map_err(|code| code.as_str().to_string()),
        }
    }

    pub(crate) fn manager(&self) -> OllamaManager {
        self.manager.clone()
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
        let coordinator = crate::app_exit::AppExitCoordinator::initialize()
            .map_err(|_| "failed to initialize Ollama test manager".to_string())?;
        Ok(Self::build(
            OllamaManager::new(coordinator.work_supervisor()),
            Some(base_url.trim_end_matches('/').to_string()),
        ))
    }

    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!(
                "{}/api/tags",
                match self.base_url().await {
                    Ok(url) => url,
                    Err(_) => return false,
                }
            ))
            .timeout(TIMEOUT)
            .send()
            .await
            .is_ok()
    }

    pub async fn loaded_context_length(&self, name: &str) -> Result<Option<u64>, String> {
        model_customizations::validate_model_name(name)?;
        let base_url = self.base_url().await?;
        let resp = self
            .client
            .get(format!("{base_url}/api/ps"))
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
        let base_url = self.base_url().await?;
        let resp = self
            .client
            .get(format!("{base_url}/api/tags"))
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

    pub async fn get_model_editor_data(&self, name: &str) -> Result<OllamaModelEditorData, String> {
        let info = self.show_model(name).await?;
        let decoded =
            super::ollama_parameter_summary::parse(&info.parameters).and_then(|entries| {
                super::ollama_parameter_validation::validate_parameter_entries(&entries)?;
                Ok(entries
                    .into_iter()
                    .map(|(key, value)| OllamaParameter { key, value })
                    .collect())
            });
        let (parameters, parameter_error) = match decoded {
            Ok(parameters) => (Some(parameters), None),
            Err(error) => (None, Some(error)),
        };
        Ok(OllamaModelEditorData {
            modelfile: info.modelfile,
            parameters,
            parameter_error,
            prompt_tier: super::model_size::detect_ollama_tier(&info.parameter_size, name),
        })
    }

    pub async fn update_modelfile(&self, name: &str, content: &str) -> Result<(), String> {
        super::ollama_modelfile_create::create_from_modelfile(self, name, content).await
    }

    pub async fn update_parameters(
        &self,
        name: &str,
        entries: Vec<(String, String)>,
    ) -> Result<(), String> {
        let info = self.show_model(name).await?;
        let current_entries = super::ollama_parameter_summary::parse(&info.parameters)?;
        super::ollama_parameter_validation::validate_parameter_entries(&current_entries)?;
        let updated = super::ollama_modelfile_parameters::rewrite(
            &info.modelfile,
            &current_entries,
            &entries,
        )?;
        super::ollama_modelfile_create::create_from_modelfile(self, name, &updated).await
    }

    pub async fn show_model(&self, name: &str) -> Result<ModelInfo, String> {
        let base_url = self.base_url().await?;
        let resp = self
            .client
            .post(format!("{base_url}/api/show"))
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
