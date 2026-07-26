use crate::services::paths::data_dir;
use std::path::{Component, Path, PathBuf};

pub(super) struct ManagedWorktreeIdentity {
    pub path: PathBuf,
    pub root: PathBuf,
    pub child_id: String,
    pub execution_id: String,
}

impl ManagedWorktreeIdentity {
    pub fn parse(worktree_path: &str) -> Result<Self, String> {
        if worktree_path.contains('\0') {
            return Err(invalid());
        }
        let root = data_dir().join("subagent-worktrees");
        let path = PathBuf::from(worktree_path);
        let relative = path.strip_prefix(&root).map_err(|_| invalid())?;
        let mut components = relative.components();
        let child_id = normal_uuid(components.next())?;
        let execution_id = normal_uuid(components.next())?;
        if components.next().is_some() || path != root.join(&child_id).join(&execution_id) {
            return Err(invalid());
        }
        Ok(Self {
            path,
            root,
            child_id,
            execution_id,
        })
    }

    pub fn require_owner(&self, child_id: &str, execution_id: &str) -> Result<(), String> {
        if uuid::Uuid::parse_str(child_id).is_err()
            || uuid::Uuid::parse_str(execution_id).is_err()
            || self.child_id != child_id
            || self.execution_id != execution_id
        {
            return Err(invalid());
        }
        Ok(())
    }

    pub fn require_child(&self, child_id: &str) -> Result<(), String> {
        if uuid::Uuid::parse_str(child_id).is_err() || self.child_id != child_id {
            return Err(invalid());
        }
        Ok(())
    }

    pub async fn reject_symlinks(&self) -> Result<(), String> {
        reject_symlink(self.path.parent().ok_or_else(invalid)?).await?;
        reject_symlink(&self.path).await
    }
}

fn normal_uuid(component: Option<Component<'_>>) -> Result<String, String> {
    let Component::Normal(value) = component.ok_or_else(invalid)? else {
        return Err(invalid());
    };
    let value = value.to_str().ok_or_else(invalid)?;
    uuid::Uuid::parse_str(value).map_err(|_| invalid())?;
    Ok(value.to_string())
}

async fn reject_symlink(path: &Path) -> Result<(), String> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(invalid()),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(invalid()),
    }
}

fn invalid() -> String {
    "Chemin worktree invalide".to_string()
}
