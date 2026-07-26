use crate::services::agent_local::ollama_client::OllamaClient;
use crate::services::agent_local::ollama_registry_details;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModelUpdate {
    pub full_name: String,
    pub family: String,
    pub tag: String,
}

#[tauri::command]
pub async fn check_ollama_updates(
    ollama: tauri::State<'_, OllamaClient>,
) -> Result<Vec<OllamaModelUpdate>, String> {
    let models = ollama.list_models().await?;
    if models.is_empty() {
        return Ok(vec![]);
    }

    let mut families = HashSet::new();
    for m in &models {
        if families.len() >= 100 {
            break;
        }
        if let Some(fam) = m.name.split(':').next() {
            families.insert(fam.to_string());
        }
    }

    let mut updates = Vec::new();

    for family in families.iter().take(100) {
        let tags = match ollama_registry_details::fetch_model_tags(family).await {
            Ok(t) => t,
            Err(_) => continue,
        };

        for model in &models {
            let Some(fam) = model.name.split(':').next() else {
                continue;
            };
            if fam != family {
                continue;
            }
            let tag_name = model.name.split(':').nth(1).unwrap_or("latest");

            if model.is_customized {
                continue;
            }
            if let Some(rtag) = tags.iter().find(|t| t.name == tag_name) {
                if model.digest_short != rtag.digest_short {
                    updates.push(OllamaModelUpdate {
                        full_name: model.name.clone(),
                        family: family.clone(),
                        tag: tag_name.to_string(),
                    });
                }
            }
        }
    }

    Ok(updates)
}
