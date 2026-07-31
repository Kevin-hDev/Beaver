use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use zeroize::Zeroize;

const MEMORY_THRESHOLD_BYTES: usize = 28 * 1024;

pub struct ShellOutputStore {
    file: Option<tokio::fs::File>,
    buffered: Vec<u8>,
    relative_path: String,
    cleanup_file: bool,
}

impl ShellOutputStore {
    pub fn prepare(session_id: &str) -> Result<Self, String> {
        super::session_store::validate_session_id(session_id)?;
        let file_name = format!("shell-{}.log", uuid::Uuid::new_v4());
        let relative_path = PathBuf::from("tool-results")
            .join(session_id)
            .join(file_name)
            .to_string_lossy()
            .to_string();
        Ok(Self {
            file: None,
            buffered: Vec::new(),
            relative_path,
            cleanup_file: false,
        })
    }

    pub async fn append(&mut self, bytes: &[u8]) -> Result<(), String> {
        if let Some(file) = self.file.as_mut() {
            return file
                .write_all(bytes)
                .await
                .map_err(|_| "Sortie shell indisponible.".to_string());
        }
        if self.buffered.len().saturating_add(bytes.len()) <= MEMORY_THRESHOLD_BYTES {
            self.buffered.extend_from_slice(bytes);
            return Ok(());
        }
        self.activate(bytes).await
    }

    pub async fn finalize(mut self, keep: bool) -> Result<Option<String>, String> {
        self.buffered.zeroize();
        let Some(mut file) = self.file.take() else {
            return Ok(None);
        };
        file.flush()
            .await
            .map_err(|_| "Sortie shell indisponible.".to_string())?;
        drop(file);
        if keep {
            self.cleanup_file = false;
            return Ok(Some(self.relative_path.clone()));
        }
        tokio::fs::remove_file(self.absolute_path())
            .await
            .map_err(|_| "Sortie shell indisponible.".to_string())?;
        self.cleanup_file = false;
        Ok(None)
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    async fn activate(&mut self, bytes: &[u8]) -> Result<(), String> {
        let path = self.absolute_path();
        let directory = path
            .parent()
            .ok_or_else(|| "Sortie shell indisponible.".to_string())?;
        crate::services::private_store::ensure_private_dir_async(directory.to_path_buf()).await?;
        let mut options = tokio::fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options
            .open(&path)
            .await
            .map_err(|_| "Sortie shell indisponible.".to_string())?;
        let result = async {
            file.write_all(&self.buffered).await?;
            file.write_all(bytes).await
        }
        .await;
        self.buffered.zeroize();
        if result.is_err() {
            drop(file);
            let _ = tokio::fs::remove_file(path).await;
            return Err("Sortie shell indisponible.".to_string());
        }
        self.file = Some(file);
        self.cleanup_file = true;
        Ok(())
    }

    fn absolute_path(&self) -> PathBuf {
        crate::services::paths::data_dir().join(&self.relative_path)
    }
}

impl Drop for ShellOutputStore {
    fn drop(&mut self) {
        self.buffered.zeroize();
        if self.cleanup_file {
            drop(self.file.take());
            let _ = std::fs::remove_file(self.absolute_path());
        }
    }
}
