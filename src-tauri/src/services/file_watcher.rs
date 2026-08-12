use notify::{Event, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const EVENT_CONFIG: &str = "fs:config-changed";
const EVENT_PERSONALITY: &str = "fs:personality-changed";
const EVENT_LOGS: &str = "fs:logs-changed";
const EVENT_CONNECTORS: &str = "fs:connectors-changed";
const EVENT_SKILLS: &str = "fs:skills-changed";
const EVENT_PROVIDERS: &str = "fs:providers-changed";
const DEBOUNCE_MS: u64 = 200;
const MAX_DEBOUNCED_PATHS: usize = 256;

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
    let background = app
        .state::<crate::services::runtime_background::RuntimeBackgroundServices>()
        .inner()
        .clone();

    if background
        .spawn_loop(move |cancel| async move {
            let _ = tokio::task::spawn_blocking(move || run(handle, watch_paths, cancel)).await;
        })
        .is_err()
    {
        ::log::warn!("[file_watcher] unavailable during shutdown");
    }
}

fn run(
    handle: AppHandle,
    watch_paths: Vec<(PathBuf, RecursiveMode)>,
    cancel: crate::services::work_registry::ServiceWorkCancellation,
) {
    let (tx, rx) = mpsc::channel::<Event>();

    let mut watcher = match notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            ::log::error!("[file_watcher] failed to create watcher: {e}");
            return;
        }
    };

    for (path, mode) in &watch_paths {
        if path.exists() && watcher.watch(path, *mode).is_err() {
            ::log::warn!("[file_watcher] watch registration failed");
        }
    }

    while !cancel.is_cancelled() {
        let first = match rx.recv_timeout(Duration::from_millis(DEBOUNCE_MS)) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        };
        std::thread::sleep(Duration::from_millis(DEBOUNCE_MS));
        let mut all_paths = first
            .paths
            .into_iter()
            .take(MAX_DEBOUNCED_PATHS)
            .collect::<Vec<_>>();
        while let Ok(extra) = rx.try_recv() {
            let remaining = MAX_DEBOUNCED_PATHS.saturating_sub(all_paths.len());
            all_paths.extend(extra.paths.into_iter().take(remaining));
            if all_paths.len() == MAX_DEBOUNCED_PATHS {
                break;
            }
        }

        let mut emitted: HashSet<&str> = HashSet::new();
        for changed in &all_paths {
            let normalized = normalize_path(changed);
            if let Some(event_name) = classify_path(&normalized) {
                if emitted.insert(event_name) {
                    let _ = handle.emit(event_name, ());
                    if event_name == EVENT_CONFIG {
                        let app = handle.clone();
                        let background = handle
                            .state::<crate::services::runtime_background::RuntimeBackgroundServices>()
                            .inner()
                            .clone();
                        let _ = background.spawn_task(move |cancel| async move {
                            tokio::select! {
                                _ = cancel.cancelled() => {}
                                _ = crate::services::mascot::sync_from_disk(app) => {}
                            }
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "file_watcher_tests.rs"]
mod tests;
