use std::path::Component;
use std::sync::Mutex;
use std::time::Duration;

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use tauri::{AppHandle, Emitter, State};

use super::file_tree::{validate_path, HIDDEN_ENTRIES};

pub struct FileTreeWatcher {
    inner: Mutex<Option<WatcherState>>,
}

struct WatcherState {
    _debouncer: Debouncer<notify::RecommendedWatcher>,
    _path: String,
}

impl FileTreeWatcher {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
}

#[derive(Clone, serde::Serialize)]
struct FileTreeChangedPayload {
    path: String,
    kind: String,
}

const DEBOUNCE_MS: u64 = 200;
const MAX_EVENTS_PER_BATCH: usize = 50;

fn has_hidden_segment(path: &std::path::Path) -> bool {
    path.components().any(|c| {
        if let Component::Normal(s) = c {
            HIDDEN_ENTRIES.contains(&s.to_string_lossy().as_ref())
        } else {
            false
        }
    })
}

#[tauri::command]
pub fn watch_project_directory(
    path: String,
    app: AppHandle,
    state: State<'_, FileTreeWatcher>,
) -> Result<(), String> {
    validate_path(&path)?;

    let canonical = std::fs::canonicalize(&path).map_err(|_| "Dossier introuvable".to_string())?;

    if !canonical.is_dir() {
        return Err("Dossier introuvable".into());
    }

    let canonical_str = canonical.to_string_lossy().to_string();

    let mut guard = state
        .inner
        .lock()
        .map_err(|_| "Erreur interne".to_string())?;

    *guard = None;

    let app_handle = app.clone();
    let watched_path = canonical_str.clone();
    let watched_root = canonical.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(DEBOUNCE_MS),
        move |res: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            let events = match res {
                Ok(e) => e,
                Err(_) => return,
            };

            let mut emitted = std::collections::HashSet::new();

            for event in events.iter().take(MAX_EVENTS_PER_BATCH) {
                if matches!(event.kind, DebouncedEventKind::AnyContinuous) {
                    continue;
                }

                if !event.path.starts_with(&watched_root) {
                    continue;
                }

                if has_hidden_segment(&event.path) {
                    continue;
                }

                let parent = event
                    .path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| watched_path.clone());

                if emitted.insert(parent.clone()) {
                    let _ = app_handle.emit(
                        "file-tree-changed",
                        FileTreeChangedPayload {
                            path: parent,
                            kind: "changed".to_string(),
                        },
                    );
                }
            }
        },
    )
    .map_err(|_| "Impossible de surveiller ce dossier".to_string())?;

    debouncer
        .watcher()
        .watch(&canonical, RecursiveMode::Recursive)
        .map_err(|_| "Impossible de surveiller ce dossier".to_string())?;

    *guard = Some(WatcherState {
        _debouncer: debouncer,
        _path: canonical_str,
    });

    Ok(())
}

#[tauri::command]
pub fn unwatch_project_directory(state: State<'_, FileTreeWatcher>) -> Result<(), String> {
    let mut guard = state
        .inner
        .lock()
        .map_err(|_| "Erreur interne".to_string())?;
    *guard = None;
    Ok(())
}
