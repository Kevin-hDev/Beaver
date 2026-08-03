use crate::services::codex_oauth::{jwt, login, store};
use crate::services::llm::types::ModelInfo;
use tauri::Emitter;

#[tauri::command]
pub async fn codex_login(app: tauri::AppHandle) -> Result<String, String> {
    let result = login::login().await;
    if result.is_ok() {
        crate::services::codex_client::model_catalog::invalidate().await;
        let _ = app.emit("codex-auth-changed", ());
    }
    result
}

#[tauri::command]
pub async fn codex_logout(app: tauri::AppHandle) -> Result<(), String> {
    let result = login::logout().await;
    if result.is_ok() {
        crate::services::codex_client::model_catalog::invalidate().await;
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

pub(crate) async fn resolved_codex_models() -> Result<Vec<ModelInfo>, String> {
    crate::services::codex_client::model_catalog::available_models().await
}

#[tauri::command]
pub async fn codex_models() -> Vec<ModelInfo> {
    resolved_codex_models()
        .await
        .unwrap_or_else(|_| crate::services::codex_client::model_catalog::fallback_models())
}

#[cfg(test)]
mod tests {
    #[test]
    fn fallback_codex_models_include_gpt_56_with_exact_modes() {
        let models = crate::services::codex_client::model_catalog::fallback_models();
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

        assert_eq!(sol.context_length, Some(258_400));
        assert_eq!(terra.context_length, Some(258_400));
        assert_eq!(luna.context_length, Some(258_400));
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
    fn fallback_codex_models_include_text_only_spark() {
        let models = crate::services::codex_client::model_catalog::fallback_models();
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
