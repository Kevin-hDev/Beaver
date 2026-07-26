use std::path::{Path, PathBuf};

pub async fn resolve(
    project_id: Option<&str>,
    child_session_id: &str,
) -> Result<PathBuf, String> {
    if let Some(project_id) = project_id {
        if let Some(project) = super::project_store::list()
            .await
            .unwrap_or_default()
            .into_iter()
            .find(|project| project.id == project_id && Path::new(&project.path).is_dir())
        {
            return PathBuf::from(project.path)
                .canonicalize()
                .map_err(|_| directory_error());
        }
    }
    let child = super::session_store::get(child_session_id)
        .await
        .map_err(|_| directory_error())?;
    let path = PathBuf::from(child.working_dir);
    if !path.is_absolute() || !path.is_dir() {
        return Err(directory_error());
    }
    path.canonicalize().map_err(|_| directory_error())
}

pub async fn for_child(child_session_id: &str) -> Result<PathBuf, String> {
    let child = super::session_store::get(child_session_id)
        .await
        .map_err(|_| directory_error())?;
    resolve(child.project_id.as_deref(), child_session_id).await
}

fn directory_error() -> String {
    "Un sous-agent code doit être lancé depuis un dossier valide.".to_string()
}
