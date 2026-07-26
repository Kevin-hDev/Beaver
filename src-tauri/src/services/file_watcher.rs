use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const EVENT_CONFIG: &str = "fs:config-changed";
const EVENT_PERSONALITY: &str = "fs:personality-changed";
const EVENT_LOGS: &str = "fs:logs-changed";
const EVENT_CONNECTORS: &str = "fs:connectors-changed";
const EVENT_SKILLS: &str = "fs:skills-changed";
const EVENT_PROVIDERS: &str = "fs:providers-changed";
const DEBOUNCE_MS: u64 = 200;

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn classify_path(n: &str) -> Option<&'static str> {
    if n.contains("memory/core") || n.contains("memory\\core") {
        return Some(EVENT_PERSONALITY);
    }
    if n.ends_with("mcp-connectors.json") {
        return Some(EVENT_CONNECTORS);
    }
    if n.ends_with("configured-providers.json") {
        return Some(EVENT_PROVIDERS);
    }
    if n.contains("/skills/")
        || n.contains("\\skills\\")
        || n.ends_with("/skills")
        || n.ends_with("\\skills")
    {
        return Some(EVENT_SKILLS);
    }
    if n.ends_with("config.json")
        || n.ends_with("favorite-models.json")
        || n.ends_with("agent-settings.json")
    {
        return Some(EVENT_CONFIG);
    }
    if n.ends_with("logs/wakeups.jsonl") || n.ends_with("logs\\wakeups.jsonl") {
        return Some(EVENT_LOGS);
    }
    if n.contains("/inbox/") || n.contains("\\inbox\\") {
        return Some(EVENT_PERSONALITY);
    }
    None
}

pub fn start(app: &AppHandle) {
    let base = crate::services::paths::data_dir();

    let watch_paths: Vec<(PathBuf, RecursiveMode)> = vec![
        (base.clone(), RecursiveMode::NonRecursive),
        (base.join("memory/core"), RecursiveMode::NonRecursive),
        (base.join("inbox"), RecursiveMode::NonRecursive),
        (base.join("logs"), RecursiveMode::NonRecursive),
        (base.join("skills"), RecursiveMode::Recursive),
    ];

    let handle = app.clone();

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<Event>();

        let mut watcher = match notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[file_watcher] failed to create watcher: {e}");
                return;
            }
        };

        for (path, mode) in &watch_paths {
            if path.exists() {
                if let Err(e) = watcher.watch(path, *mode) {
                    eprintln!("[file_watcher] watch error for {}: {e}", path.display());
                }
            }
        }

        while let Ok(first) = rx.recv() {
            thread::sleep(Duration::from_millis(DEBOUNCE_MS));

            let mut all_paths: Vec<PathBuf> = first.paths;
            while let Ok(extra) = rx.try_recv() {
                all_paths.extend(extra.paths);
            }

            let mut emitted: HashSet<&str> = HashSet::new();
            for changed in &all_paths {
                let normalized = normalize_path(changed);
                if let Some(event_name) = classify_path(&normalized) {
                    if emitted.insert(event_name) {
                        let _ = handle.emit(event_name, ());
                        if event_name == EVENT_CONFIG {
                            tauri::async_runtime::spawn(crate::services::mascot::sync_from_disk(
                                handle.clone(),
                            ));
                        }
                    }
                }
            }
        }
    });
}

#[cfg(test)]
#[path = "file_watcher_tests.rs"]
mod tests;
