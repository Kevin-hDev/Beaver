use std::path::{Component, Path, PathBuf};

pub use super::memory_path_security::validate_in_scope;
pub use super::memory_project_id::valid_project_id;

#[derive(Debug, Clone)]
pub struct MemoryLayout {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MemoryScope {
    pub id: String,
    pub label: String,
    pub root: PathBuf,
}

impl MemoryLayout {
    pub fn production() -> Self {
        Self::at(crate::services::paths::data_dir().join("memory"))
    }

    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn global_scope(&self) -> MemoryScope {
        MemoryScope {
            id: "global".into(),
            label: "Global".into(),
            root: self.root.join("global"),
        }
    }

    pub async fn project_scope_ready(
        &self,
        working_dir: &Path,
    ) -> Result<MemoryScope, String> {
        super::memory_project_migration::resolve(self, working_dir).await
    }

    pub async fn scope_for_tool_path(
        &self,
        raw_path: &str,
        working_dir: &Path,
    ) -> Result<Option<MemoryScope>, String> {
        super::memory_project_migration::scope_for_tool_path(self, raw_path, working_dir).await
    }

    pub fn management_topic(
        &self,
        raw_path: &str,
    ) -> Result<(MemoryScope, PathBuf), String> {
        if !Path::new(raw_path).is_absolute() {
            return Err("Chemin mémoire invalide.".into());
        }
        let candidate = lexical_path(raw_path, &self.root)?;
        let global = self.global_scope();
        if candidate.starts_with(global.topics_dir()) {
            let resolved = validate_in_scope(&global, &candidate)?;
            return Ok((global, resolved));
        }
        let projects = self.root.join("projects");
        let relative = candidate
            .strip_prefix(&projects)
            .map_err(|_| "Chemin mémoire interdit.".to_string())?;
        let parts = relative.components().collect::<Vec<_>>();
        if parts.len() != 3
            || parts[1].as_os_str() != "topics"
            || parts[0]
                .as_os_str()
                .to_str()
                .is_none_or(|id| !valid_project_id(id))
        {
            return Err("Chemin mémoire invalide.".into());
        }
        let id = parts[0].as_os_str().to_string_lossy().into_owned();
        let scope = MemoryScope {
            id: id.clone(),
            label: id.clone(),
            root: projects.join(id),
        };
        let resolved = validate_in_scope(&scope, &candidate)?;
        Ok((scope, resolved))
    }
}

impl MemoryScope {
    pub fn topics_dir(&self) -> PathBuf {
        self.root.join("topics")
    }

    pub fn archive_dir(&self) -> PathBuf {
        self.root.join("archive")
    }

    pub fn summary_path(&self) -> PathBuf {
        self.root.join("memory_summary.md")
    }

    pub fn registry_path(&self) -> PathBuf {
        self.root.join("MEMORY.md")
    }

    pub async fn ensure(&self) -> Result<(), String> {
        super::memory_path_security::ensure_scope_dir(self).await?;
        super::memory_store::write_if_missing(&self.registry_path(), "# Registre mémoire\n").await?;
        super::memory_store::write_if_missing(&self.summary_path(), "# Résumé mémoire\n").await
    }
}

pub fn lexical_path(raw_path: &str, working_dir: &Path) -> Result<PathBuf, String> {
    if raw_path.is_empty() || raw_path.len() > 4_096 || raw_path.contains('\0') {
        return Err("Chemin mémoire invalide.".into());
    }
    let path = Path::new(raw_path);
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err("Chemin mémoire invalide.".into());
    }
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        working_dir.join(path)
    })
}

pub fn path_arg<'a>(tool_name: &str, args: &'a serde_json::Value) -> Option<&'a str> {
    match tool_name {
        "read_file" | "write_file" | "edit_file" | "list_dir" => args["path"].as_str(),
        "grep" | "glob" => args.get("path").and_then(serde_json::Value::as_str),
        _ => None,
    }
}

pub fn command_mentions_memory(command: &str) -> bool {
    let normalized = command.replace('\\', "/").to_lowercase();
    let root = crate::services::paths::data_dir()
        .join("memory")
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
    normalized.contains(&root)
        || normalized.contains(".local/share/cl-go-dash/memory")
}

#[cfg(test)]
#[path = "memory_paths_tests.rs"]
mod tests;
