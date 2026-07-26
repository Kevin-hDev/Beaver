use crate::services::agent_local::project_store::{self, Project};
use crate::services::agent_local::session_store;

#[tauri::command]
pub async fn list_projects() -> Result<Vec<Project>, String> {
    project_store::list().await
}

#[tauri::command]
pub async fn add_project(path: String) -> Result<Project, String> {
    let pb = std::path::Path::new(&path);
    if !pb.is_dir() {
        return Err("Le chemin ne pointe pas vers un dossier valide".to_string());
    }
    project_store::add(&path).await
}

#[tauri::command]
pub async fn rename_project(id: String, name: String) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Le nom ne peut pas être vide".to_string());
    }
    project_store::rename(&id, name.trim()).await
}

#[tauri::command]
pub async fn delete_project(id: String) -> Result<(), String> {
    project_store::delete(&id).await?;
    session_store::clear_project_id(&id).await
}

#[tauri::command]
pub async fn reorder_projects(ids: Vec<String>) -> Result<(), String> {
    project_store::reorder(ids).await
}

#[tauri::command]
pub async fn open_project_folder(path: String) -> Result<(), String> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        "explorer"
    };
    std::process::Command::new(cmd)
        .arg(&path)
        .spawn()
        .map_err(|e| {
            eprintln!("[projects] open_folder: {e}");
            "Impossible d'ouvrir le dossier".to_string()
        })?;
    Ok(())
}
