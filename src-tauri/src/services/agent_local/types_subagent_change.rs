use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const MAX_CHANGED_PATHS: usize = 128;
pub const MAX_CHANGED_PATH_CHARS: usize = 512;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentChangeStatus {
    Pending,
    Conflict,
    Applied,
    Discarded,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentWorkspaceKind {
    #[default]
    Git,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentChangedPath {
    pub path: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubagentChangeMeta {
    pub id: String,
    pub child_session_id: String,
    pub project_id: String,
    pub base_commit: String,
    pub commit: String,
    pub branch: String,
    pub target_branch: String,
    #[serde(default)]
    pub workspace_kind: SubagentWorkspaceKind,
    pub changed_paths: Vec<SubagentChangedPath>,
    pub paths_truncated: bool,
    pub status: SubagentChangeStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub applied_commit: Option<String>,
}

impl SubagentChangeMeta {
    pub fn validate(&self) -> Result<(), String> {
        validate_uuid(&self.id)?;
        validate_uuid(&self.child_session_id)?;
        validate_text(&self.project_id, 128)?;
        validate_oid(&self.base_commit)?;
        validate_oid(&self.commit)?;
        validate_branch(&self.branch)?;
        validate_branch(&self.target_branch)?;
        if self.changed_paths.len() > MAX_CHANGED_PATHS {
            return Err("Métadonnées de changement invalides".into());
        }
        for changed in &self.changed_paths {
            validate_path(&changed.path)?;
            validate_text(&changed.kind, 16)?;
        }
        Ok(())
    }
}

pub fn validate_uuid(value: &str) -> Result<(), String> {
    let id = uuid::Uuid::parse_str(value).map_err(|_| "Identifiant invalide".to_string())?;
    if id.get_version() == Some(uuid::Version::Random) {
        Ok(())
    } else {
        Err("Identifiant invalide".into())
    }
}

fn validate_oid(value: &str) -> Result<(), String> {
    if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("Référence Git invalide".into())
    }
}

fn validate_branch(value: &str) -> Result<(), String> {
    crate::services::git::branch::validate_branch_name(value)
        .map_err(|_| "Branche Git invalide".into())
}

fn validate_path(value: &str) -> Result<(), String> {
    validate_text(value, MAX_CHANGED_PATH_CHARS)?;
    let path = std::path::Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err("Chemin modifié invalide".into());
    }
    Ok(())
}

fn validate_text(value: &str, max: usize) -> Result<(), String> {
    if value.is_empty() || value.contains('\0') || value.chars().count() > max {
        Err("Métadonnées de changement invalides".into())
    } else {
        Ok(())
    }
}
