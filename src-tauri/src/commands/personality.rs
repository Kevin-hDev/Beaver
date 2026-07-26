use crate::services::personality_injection;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct PersonalityFile {
    pub name: String,
    pub path: String,
    pub description: String,
}

fn data_root() -> PathBuf {
    crate::services::paths::data_dir()
}

fn memory_core() -> PathBuf {
    data_root().join("memory/core")
}

fn inbox_dir() -> PathBuf {
    data_root().join("inbox")
}

const ROOT_FILES: &[(&str, &str)] = &[("AGENTS.md", "Instructions agent")];

const CORE_FILES: &[(&str, &str)] = &[
    ("identity.md", "Qui est Jackson"),
    ("principles.md", "Règles et valeurs"),
    ("user.md", "Profil de Kevin"),
];

const INBOX_FILES: &[(&str, &str)] = &[("idea-discovery.md", "Idées en attente")];

#[tauri::command]
pub fn list_personality_files() -> Result<Vec<PersonalityFile>, String> {
    let root = data_root();
    let core = memory_core();
    let inbox = inbox_dir();

    let mut files: Vec<PersonalityFile> = Vec::new();

    let sources: &[(&PathBuf, &[(&str, &str)])] = &[
        (&root, ROOT_FILES),
        (&core, CORE_FILES),
        (&inbox, INBOX_FILES),
    ];

    for (dir, entries) in sources {
        for (name, desc) in *entries {
            let path = dir.join(name);
            if path.exists() {
                files.push(PersonalityFile {
                    name: name.to_string(),
                    path: path.to_string_lossy().to_string(),
                    description: desc.to_string(),
                });
            }
        }
    }

    Ok(files)
}

#[tauri::command]
pub fn read_personality_file(path: String) -> Result<String, String> {
    let canonical = PathBuf::from(&path)
        .canonicalize()
        .map_err(|_| "Fichier introuvable".to_string())?;
    let root = data_root()
        .canonicalize()
        .map_err(|_| "Erreur système".to_string())?;
    if !canonical.starts_with(&root) {
        return Err("Chemin non autorisé".to_string());
    }
    fs::read_to_string(&canonical).map_err(|_| "Erreur de lecture".to_string())
}

#[tauri::command]
pub fn open_in_editor(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err("File not found".to_string());
    }

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(&path).spawn();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(&path).spawn();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").args(["/c", "start", "", &path]).spawn();

    result.map_err(|e| {
        eprintln!("[personality] open_in_editor: {e}");
        "Impossible d'ouvrir l'éditeur".to_string()
    })?;
    Ok(())
}

#[tauri::command]
pub fn get_injection_state() -> Result<HashMap<String, bool>, String> {
    Ok(personality_injection::read_state())
}

#[tauri::command]
pub fn set_injection_state(name: String, enabled: bool) -> Result<(), String> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Nom de fichier invalide".to_string());
    }
    let mut state = personality_injection::read_state();
    state.insert(name, enabled);
    personality_injection::write_state(&state)
}
