use crate::services::agent_local::model_customizations;
use crate::services::agent_local::ollama_client::OllamaClient;
use crate::services::agent_local::ollama_registry;
use crate::services::agent_local::ollama_registry_details;
use crate::services::agent_local::translation_cache;
use crate::services::agent_local::translator;
use crate::services::agent_local::types_ollama::{
    ModelInfo, OllamaModel, RegistryModel, RegistryModelDetails, RegistryTag,
};
use crate::services::ollama_lifecycle;
use tauri::Emitter;

#[tauri::command]
pub async fn list_ollama_models(
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<Vec<OllamaModel>, String> {
    if !ollama_lifecycle::is_ollama_ready() {
        return Ok(Vec::new());
    }
    ollama.list_models().await
}

#[tauri::command]
pub async fn show_ollama_model(
    name: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<ModelInfo, String> {
    ollama.show_model(&name).await
}

#[tauri::command]
pub async fn is_ollama_running(ollama: tauri::State<'_, OllamaClient>) -> Result<bool, String> {
    Ok(ollama.is_running().await)
}

#[tauri::command]
pub async fn get_loaded_ollama_context(
    name: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<Option<u64>, String> {
    ollama.loaded_context_length(&name).await
}

#[tauri::command]
pub async fn search_ollama_models(query: String) -> Result<Vec<RegistryModel>, String> {
    ollama_registry::search_models(&query).await
}

#[tauri::command]
pub async fn get_registry_model_details(name: String) -> Result<RegistryModelDetails, String> {
    ollama_registry_details::fetch_model_details(&name).await
}

#[tauri::command]
pub async fn list_registry_tags(name: String) -> Result<Vec<RegistryTag>, String> {
    ollama_registry_details::fetch_model_tags(&name).await
}

#[tauri::command]
pub async fn translate_description(
    model_name: String,
    text: String,
    target_lang: String,
    translator_model: Option<String>,
) -> Result<String, String> {
    if let Some(cached) = translation_cache::get_cached(&model_name, &target_lang).await {
        return Ok(cached);
    }
    let translated =
        translator::translate_text(&text, &target_lang, translator_model.as_deref()).await?;
    translation_cache::set_cached(&model_name, &target_lang, &translated).await?;
    Ok(translated)
}

#[tauri::command]
pub async fn delete_ollama_model(app: tauri::AppHandle, name: String) -> Result<(), String> {
    ollama_registry::delete_model(&name).await?;
    crate::services::agent_local::system_prompt_store::remove_ollama_model(&name)?;
    crate::services::agent_local::ollama_native_prompts::remove(&name)?;
    model_customizations::clear_model_customized(&name)?;
    let _ = app.emit("ollama-models-changed", ());
    Ok(())
}

#[tauri::command]
pub async fn get_modelfile(
    name: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<String, String> {
    ollama.get_modelfile(&name).await
}

#[tauri::command]
pub async fn update_modelfile(
    app: tauri::AppHandle,
    name: String,
    content: String,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<(), String> {
    let previous_kind = model_customizations::customization_kind(&name);
    // A full Modelfile edit can replace SYSTEM, so preserve a trustworthy
    // native prompt before the destructive operation and block on failure.
    if model_customizations::can_capture_current(previous_kind) {
        crate::services::agent_local::ollama_native_prompts::capture_current(&ollama, &name)
            .await?;
    }
    model_customizations::mark_modelfile_customized(&name)?;
    if let Err(e) = ollama.update_modelfile(&name, &content).await {
        let _ = model_customizations::restore_customization_kind(&name, previous_kind);
        return Err(e);
    }
    let _ = app.emit("modelfile-updated", &name);
    Ok(())
}

#[tauri::command]
pub async fn update_parameters(
    app: tauri::AppHandle,
    name: String,
    parameters: Vec<(String, String)>,
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<(), String> {
    let previous_kind = model_customizations::customization_kind(&name);
    // Parameter updates preserve SYSTEM. Capture is therefore best effort:
    // a ParametersOnly marker lets future reads retry safely if this fails.
    if model_customizations::can_capture_current(previous_kind) {
        let _ =
            crate::services::agent_local::ollama_native_prompts::capture_current(&ollama, &name)
                .await;
    }
    model_customizations::mark_parameters_customized(&name)?;
    if let Err(e) = ollama.update_parameters(&name, parameters).await {
        let _ = model_customizations::restore_customization_kind(&name, previous_kind);
        return Err(e);
    }
    let _ = app.emit("modelfile-updated", &name);
    Ok(())
}
