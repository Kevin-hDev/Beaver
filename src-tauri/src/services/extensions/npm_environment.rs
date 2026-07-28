use rand::RngCore;
use std::path::{Path, PathBuf};

pub struct ProjectConfig {
    original: PathBuf,
    held: Option<PathBuf>,
}

impl ProjectConfig {
    pub fn neutralize(root: &Path) -> Result<Self, String> {
        let original = root.join(".npmrc");
        if !original.exists() {
            return Ok(Self {
                original,
                held: None,
            });
        }
        let metadata = std::fs::symlink_metadata(&original)
            .map_err(|_| "Configuration npm invalide.".to_string())?;
        if !metadata.file_type().is_file()
            || metadata.len() > super::types::MAX_MESSAGE_BYTES as u64
        {
            return Err("Configuration npm invalide.".to_string());
        }
        let parent = root
            .parent()
            .ok_or_else(|| "Configuration npm invalide.".to_string())?;
        let mut random = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut random);
        let held = parent.join(format!(".npmrc-{}.held", hex::encode(random)));
        std::fs::rename(&original, &held)
            .map_err(|_| "Configuration npm impossible à isoler.".to_string())?;
        Ok(Self {
            original,
            held: Some(held),
        })
    }

    pub fn restore(mut self) -> Result<(), String> {
        self.restore_inner()
    }

    fn restore_inner(&mut self) -> Result<(), String> {
        let Some(held) = self.held.as_ref() else {
            return Ok(());
        };
        if self.original.exists() {
            let metadata = std::fs::symlink_metadata(&self.original)
                .map_err(|_| "Configuration npm impossible à restaurer.".to_string())?;
            if metadata.file_type().is_dir() {
                return Err("Configuration npm impossible à restaurer.".to_string());
            }
            std::fs::remove_file(&self.original)
                .map_err(|_| "Configuration npm impossible à restaurer.".to_string())?;
        }
        std::fs::rename(held, &self.original)
            .map_err(|_| "Configuration npm impossible à restaurer.".to_string())?;
        self.held = None;
        Ok(())
    }
}

impl Drop for ProjectConfig {
    fn drop(&mut self) {
        let _ = self.restore_inner();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_configuration_is_hidden_then_restored() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("extension");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join(".npmrc"), "registry=http://unsafe.invalid").unwrap();

        let guard = ProjectConfig::neutralize(&root).unwrap();
        assert!(!root.join(".npmrc").exists());
        guard.restore().unwrap();

        assert_eq!(
            std::fs::read_to_string(root.join(".npmrc")).unwrap(),
            "registry=http://unsafe.invalid"
        );
    }
}
