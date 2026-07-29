use super::types::{ExtensionOriginKind, ExtensionRecord};
use rand::RngCore;
use std::path::{Component, Path, PathBuf};

const DIRECTORY: &str = "extension-installs";

pub struct StagingDirectory {
    root: PathBuf,
    path: PathBuf,
    token: String,
    committed: bool,
}

pub fn prepare() -> Result<StagingDirectory, String> {
    let root = root();
    crate::services::private_store::ensure_private_dir(&root)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    let mut random = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut random);
    let token = hex::encode(random);
    let path = root.join(format!(".staging-{token}"));
    std::fs::create_dir(&path).map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    crate::services::private_store::repair_path(&path)
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    Ok(StagingDirectory {
        root,
        path,
        token,
        committed: false,
    })
}

pub fn root() -> PathBuf {
    crate::services::paths::data_dir().join(DIRECTORY)
}

impl StagingDirectory {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn commit(mut self, extension_id: &str) -> Result<PathBuf, String> {
        super::validation::identifier(extension_id)?;
        let parent = self.root.join(extension_id);
        crate::services::private_store::ensure_private_dir(&parent)
            .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
        let destination = parent.join(&self.token);
        std::fs::rename(&self.path, &destination)
            .map_err(|_| "Installation d'extension impossible.".to_string())?;
        self.committed = true;
        Ok(destination)
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

pub fn rewrite_source(
    record: &mut ExtensionRecord,
    staging: &Path,
    installed: &Path,
) -> Result<(), String> {
    let relative = Path::new(&record.source)
        .strip_prefix(staging)
        .map_err(|_| "Source d'extension gérée invalide.".to_string())?;
    record.source = installed
        .join(relative)
        .to_str()
        .ok_or_else(|| "Source d'extension gérée invalide.".to_string())?
        .to_string();
    Ok(())
}

pub fn remove_record(record: &ExtensionRecord) -> Result<(), String> {
    let install = install_root(record)?;
    if !install.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&install)
        .map_err(|_| "Suppression des fichiers d'extension impossible.".to_string())?;
    if let Some(parent) = install.parent() {
        let is_empty = std::fs::read_dir(parent)
            .map_err(|_| "Stockage des extensions indisponible.".to_string())?
            .next()
            .is_none();
        if is_empty {
            std::fs::remove_dir(parent)
                .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
        }
    }
    Ok(())
}

pub(super) fn install_root(record: &ExtensionRecord) -> Result<PathBuf, String> {
    if !record.origin.as_ref().is_some_and(|origin| {
        matches!(
            origin.kind,
            ExtensionOriginKind::Git | ExtensionOriginKind::Npm
        )
    }) {
        return Err("Extension non gérée.".to_string());
    }
    let root = root()
        .canonicalize()
        .map_err(|_| "Stockage des extensions indisponible.".to_string())?;
    let requested = PathBuf::from(&record.source);
    let source = if requested.exists() {
        requested
            .canonicalize()
            .map_err(|_| "Source d'extension gérée invalide.".to_string())?
    } else {
        requested
    };
    let relative = source
        .strip_prefix(&root)
        .map_err(|_| "Source d'extension gérée invalide.".to_string())?;
    let mut components = relative.components();
    let identifier = normal_component(components.next())?;
    let token = normal_component(components.next())?;
    if identifier != record.manifest.id || !valid_token(token) {
        return Err("Source d'extension gérée invalide.".to_string());
    }
    let install = root.join(identifier).join(token);
    if !source.starts_with(&install) {
        return Err("Source d'extension gérée invalide.".to_string());
    }
    Ok(install)
}

fn normal_component(component: Option<Component<'_>>) -> Result<&str, String> {
    match component {
        Some(Component::Normal(value)) => value
            .to_str()
            .ok_or_else(|| "Source d'extension gérée invalide.".to_string()),
        _ => Err("Source d'extension gérée invalide.".to_string()),
    }
}

fn valid_token(value: &str) -> bool {
    value.len() == 32
        && value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}
