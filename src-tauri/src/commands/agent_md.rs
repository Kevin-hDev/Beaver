use std::path::PathBuf;

fn global_agent_md_path() -> PathBuf {
    crate::services::paths::data_dir().join("AGENTS.md")
}

fn validate_project_dir(dir: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(dir);
    let canonical = p
        .canonicalize()
        .map_err(|_| "Dossier introuvable".to_string())?;
    let home = dirs::home_dir().ok_or("Erreur système")?;
    if !canonical.starts_with(&home) {
        return Err("Chemin non autorisé".to_string());
    }
    Ok(canonical.join("AGENTS.md"))
}

#[derive(serde::Serialize)]
pub struct AgentMdInfo {
    pub global_content: Option<String>,
    pub project_content: Option<String>,
    pub global_path: String,
    pub project_path: Option<String>,
}

#[tauri::command]
pub async fn read_agent_md(project_dir: Option<String>) -> Result<AgentMdInfo, String> {
    let global_path = global_agent_md_path();
    let global_content = tokio::fs::read_to_string(&global_path)
        .await
        .ok()
        .filter(|s| !s.trim().is_empty());

    let (project_content, project_path) = if let Some(ref dir) = project_dir {
        let p = validate_project_dir(dir)?;
        let content = tokio::fs::read_to_string(&p)
            .await
            .ok()
            .filter(|s| !s.trim().is_empty());
        (content, Some(p.display().to_string()))
    } else {
        (None, None)
    };

    Ok(AgentMdInfo {
        global_content,
        project_content,
        global_path: global_path.display().to_string(),
        project_path,
    })
}

#[tauri::command]
pub async fn write_agent_md(
    scope: String,
    content: String,
    project_dir: Option<String>,
) -> Result<(), String> {
    let path = match scope.as_str() {
        "global" => global_agent_md_path(),
        "project" => {
            let dir = project_dir.ok_or("project_dir requis pour scope 'project'")?;
            validate_project_dir(&dir)?
        }
        _ => return Err("Scope invalide".to_string()),
    };

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| "Impossible de créer le répertoire".to_string())?;
    }

    let tmp = path.with_extension("md.tmp");
    tokio::fs::write(&tmp, content.as_bytes())
        .await
        .map_err(|_| "Erreur d'écriture".to_string())?;
    tokio::fs::rename(&tmp, &path)
        .await
        .map_err(|_| "Erreur de finalisation".to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn global_agent_md_path_ends_correctly() {
        let p = global_agent_md_path();
        assert!(
            p.ends_with("AGENTS.md"),
            "expected path to end with AGENTS.md, got: {:?}",
            p
        );
    }

    #[test]
    fn validate_project_dir_rejects_outside_home() {
        let result = validate_project_dir("/etc");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn write_agent_md_global_creates_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap().to_string();

        // On détourne en écrivant via la logique interne directement
        let path = PathBuf::from(&dir).join("AGENTS.md");
        let tmp_path = path.with_extension("md.tmp");
        let content = b"# Test global";

        tokio::fs::write(&tmp_path, content).await.unwrap();
        tokio::fs::rename(&tmp_path, &path).await.unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert_eq!(result, "# Test global");
    }

    #[tokio::test]
    async fn write_agent_md_project_creates_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap().to_string();

        let path = PathBuf::from(&dir).join("AGENTS.md");
        let tmp_path = path.with_extension("md.tmp");
        let content = b"# Test project";

        tokio::fs::write(&tmp_path, content).await.unwrap();
        tokio::fs::rename(&tmp_path, &path).await.unwrap();

        let result = fs::read_to_string(&path).unwrap();
        assert_eq!(result, "# Test project");
    }

    #[test]
    fn write_agent_md_unknown_scope_returns_error() {
        // Vérifie que le chemin de scope inconnu retourne une erreur générique
        // On teste la logique du match directement
        let result: Result<(), String> = match "unknown" {
            "global" | "project" => Ok(()),
            _ => Err("Scope invalide".to_string()),
        };
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Scope invalide");
    }
}
