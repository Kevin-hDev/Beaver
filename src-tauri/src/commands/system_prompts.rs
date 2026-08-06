use crate::services::agent_local::ollama_client::OllamaClient;
use crate::services::agent_local::system_prompt_resolver::{resolve_global, resolve_ollama};
use crate::services::agent_local::system_prompt_store;
use crate::services::agent_local::system_prompt_types::{PromptMode, PromptTier, SystemPromptView};
use serde::Deserialize;
use tauri::Emitter;

#[derive(Clone, Deserialize)]
#[serde(tag = "scope", rename_all = "lowercase")]
pub enum SystemPromptTarget {
    Global,
    Ollama { model: String },
}

#[tauri::command]
pub async fn get_system_prompt_setting(
    target: SystemPromptTarget,
    mode: PromptMode,
    tier: PromptTier,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<SystemPromptView, String> {
    resolve_view(&target, mode, tier, &ollama).await
}

#[tauri::command]
pub async fn save_system_prompt_setting(
    app: tauri::AppHandle,
    target: SystemPromptTarget,
    mode: PromptMode,
    tier: PromptTier,
    content: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<SystemPromptView, String> {
    match &target {
        SystemPromptTarget::Global => system_prompt_store::save_global(mode, tier, &content)?,
        SystemPromptTarget::Ollama { model } => {
            system_prompt_store::save_ollama(model, mode, tier, &content)?;
        }
    }
    let view = resolve_view(&target, mode, tier, &ollama).await?;
    let _ = app.emit("system-prompts-changed", ());
    Ok(view)
}

#[tauri::command]
pub async fn restore_system_prompt_setting(
    app: tauri::AppHandle,
    target: SystemPromptTarget,
    mode: PromptMode,
    tier: PromptTier,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<SystemPromptView, String> {
    match &target {
        SystemPromptTarget::Global => system_prompt_store::restore_global(mode, tier)?,
        SystemPromptTarget::Ollama { model } => {
            system_prompt_store::restore_ollama(model, mode, tier)?;
        }
    }
    let view = resolve_view(&target, mode, tier, &ollama).await?;
    let _ = app.emit("system-prompts-changed", ());
    Ok(view)
}

async fn resolve_view(
    target: &SystemPromptTarget,
    mode: PromptMode,
    tier: PromptTier,
    ollama: &OllamaClient,
) -> Result<SystemPromptView, String> {
    let settings = system_prompt_store::snapshot()?;
    let beaver_prompt =
        crate::services::agent_local::system_prompt_defaults::beaver_prompt(mode, tier);
    match target {
        SystemPromptTarget::Global => Ok(resolve_global(&settings, mode, tier, &beaver_prompt)),
        SystemPromptTarget::Ollama { model } => {
            crate::services::agent_local::model_customizations::validate_model_name(model)?;
            let native_prompt = ollama.get_native_system_prompt(model).await?;
            Ok(resolve_ollama(
                &settings,
                model,
                mode,
                tier,
                native_prompt.as_deref(),
                &beaver_prompt,
            ))
        }
    }
}
