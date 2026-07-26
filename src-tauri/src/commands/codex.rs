use crate::services::codex_client::types::CODEX_MODELS;
use crate::services::codex_oauth::{jwt, login, store};
use crate::services::llm::types::ModelInfo;
use tauri::Emitter;

#[tauri::command]
pub async fn codex_login(app: tauri::AppHandle) -> Result<String, String> {
    let result = login::login().await;
    if result.is_ok() {
        let _ = app.emit("codex-auth-changed", ());
    }
    result
}

#[tauri::command]
pub fn codex_logout(app: tauri::AppHandle) -> Result<(), String> {
    let result = login::logout();
    if result.is_ok() {
        let _ = app.emit("codex-auth-changed", ());
    }
    result
}

#[tauri::command]
pub fn codex_status() -> Result<CodexStatus, String> {
    let logged_in = store::is_logged_in();
    let email = if logged_in {
        store::load()?
            .and_then(|t| jwt::extract_display_claims(&t.access).ok())
            .and_then(|c| c.email)
    } else {
        None
    };
    Ok(CodexStatus { logged_in, email })
}

#[derive(serde::Serialize)]
pub struct CodexStatus {
    pub logged_in: bool,
    pub email: Option<String>,
}

#[tauri::command]
pub fn codex_models() -> Vec<ModelInfo> {
    CODEX_MODELS
        .iter()
        .map(|spec| ModelInfo {
            id: spec.id.to_string(),
            display_name: None,
            owned_by: Some("openai".to_string()),
            context_length: Some(spec.context_length),
            supports_tools: true,
            supports_vision: spec.supports_vision,
            supports_thinking: true,
            reasoning_modes: spec
                .reasoning_modes
                .iter()
                .map(|mode| mode.to_string())
                .collect(),
            default_reasoning_mode: None,
            is_free: true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_models_include_gpt_56_with_exact_modes() {
        let models = codex_models();
        let sol = models
            .iter()
            .find(|model| model.id == "gpt-5.6-sol")
            .unwrap();
        let terra = models
            .iter()
            .find(|model| model.id == "gpt-5.6-terra")
            .unwrap();
        let luna = models
            .iter()
            .find(|model| model.id == "gpt-5.6-luna")
            .unwrap();

        assert_eq!(sol.context_length, Some(372_000));
        assert_eq!(terra.context_length, Some(372_000));
        assert_eq!(luna.context_length, Some(372_000));
        assert_eq!(
            sol.reasoning_modes,
            ["low", "medium", "high", "xhigh", "max", "ultra"]
        );
        assert_eq!(terra.reasoning_modes, sol.reasoning_modes);
        assert_eq!(
            luna.reasoning_modes,
            ["low", "medium", "high", "xhigh", "max"]
        );
    }

    #[test]
    fn codex_models_include_text_only_spark() {
        let models = codex_models();
        let spark = models
            .iter()
            .find(|model| model.id == "gpt-5.3-codex-spark")
            .unwrap();

        assert_eq!(spark.context_length, Some(128_000));
        assert!(spark.supports_tools);
        assert!(!spark.supports_vision);
        assert!(spark.supports_thinking);
        assert_eq!(spark.reasoning_modes, ["low", "medium", "high", "xhigh"]);
    }
}
