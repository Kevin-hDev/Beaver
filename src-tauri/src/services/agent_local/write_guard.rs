use std::path::{Path, PathBuf};

const MAX_READ_PATHS: usize = 1000;
const EVICT_COUNT: usize = 100;

pub struct WriteGuard {
    read_paths: Vec<PathBuf>,
}

impl WriteGuard {
    pub fn new() -> Self {
        Self {
            read_paths: Vec::new(),
        }
    }

    /// Enregistre qu'un fichier a été lu dans cette session.
    /// Collection bornée à MAX_READ_PATHS avec éviction FIFO des plus anciens.
    pub fn record_read(&mut self, path: &Path) {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if canonical.is_dir() {
            // Un dossier n'est pas un fichier éditable : on ignore.
            return;
        }
        if self.read_paths.contains(&canonical) {
            return;
        }
        if self.read_paths.len() >= MAX_READ_PATHS {
            // Éviction FIFO : supprimer les EVICT_COUNT entrées les plus anciennes
            self.read_paths.drain(..EVICT_COUNT);
        }
        self.read_paths.push(canonical);
    }

    /// Enregistre plusieurs chemins lus d'un coup (résultat de grep/glob/list_dir).
    pub fn record_reads(&mut self, paths: &[PathBuf]) {
        for p in paths {
            self.record_read(p);
        }
    }

    pub fn check_write(&self, path: &Path) -> Result<(), String> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if !path.exists() || self.read_paths.contains(&canonical) {
            return Ok(());
        }
        Err("Écriture bloquée : fichier non lu avant modification. Utilise read_file sur ce chemin d'abord.".to_string())
    }
}
