use super::{repo as git_repo, status};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const DEBOUNCE_MS: u64 = 200;
const WORKTREE_POLL_MS: u64 = 3_000;

static WATCHER_ACTIVE: std::sync::Mutex<Option<ActiveWatcher>> = std::sync::Mutex::new(None);

pub(super) struct ActiveWatcher {
    pub(super) repo_path: PathBuf,
    pub(super) stop: Arc<AtomicBool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GitWatchState {
    pub(super) head_name: Option<String>,
    pub(super) head_oid: Option<git2::Oid>,
    pub(super) dirty_count: usize,
    pub(super) worktree_signature: u64,
}

fn resolve_git_dir(repo_path: &Path) -> Option<PathBuf> {
    let dot_git = repo_path.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }
    if dot_git.is_file() {
        let content = std::fs::read_to_string(&dot_git).ok()?;
        let gitdir = parse_gitdir_content(&content)?;
        let resolved = if std::path::Path::new(&gitdir).is_absolute() {
            gitdir
        } else {
            repo_path.join(gitdir)
        };
        if resolved.is_dir() {
            return Some(resolved);
        }
    }
    None
}

/// Parse le contenu d'un fichier `.git` de worktree linked.
/// Format attendu : `gitdir: /chemin/vers/.git/worktrees/<name>`.
/// Retourne le chemin brut (absolu ou relatif tel qu'écrit), sans résoudre.
pub(super) fn parse_gitdir_content(content: &str) -> Option<PathBuf> {
    let gitdir = content.trim().strip_prefix("gitdir: ")?;
    if gitdir.is_empty() {
        return None;
    }
    Some(PathBuf::from(gitdir))
}

pub(super) fn read_watch_state(repo_path: &Path) -> Option<GitWatchState> {
    let repo = git_repo::open(repo_path).ok()?;
    let head = repo.head().ok();
    let (dirty_count, worktree_signature) = status::watch_signature(&repo).ok()?;
    Some(GitWatchState {
        head_name: head
            .as_ref()
            .and_then(|reference| reference.shorthand().ok().map(str::to_string)),
        head_oid: head.as_ref().and_then(|reference| reference.target()),
        dirty_count,
        worktree_signature,
    })
}

pub(super) fn update_watch_state(
    previous: &mut Option<GitWatchState>,
    current: Option<GitWatchState>,
) -> bool {
    let Some(current) = current else {
        return false;
    };
    let changed = previous.as_ref().is_some_and(|value| value != &current);
    *previous = Some(current);
    changed
}

pub fn setup_git_watcher(app: AppHandle, repo_path: PathBuf) -> Result<(), String> {
    let git_dir = match resolve_git_dir(&repo_path) {
        Some(d) => d,
        None => return Ok(()),
    };

    let mut guard = WATCHER_ACTIVE
        .lock()
        .map_err(|_| "Lock error".to_string())?;

    if let Some(previous) = guard.take() {
        previous.stop.store(true, Ordering::Relaxed);
    }

    let stop = Arc::new(AtomicBool::new(false));
    *guard = Some(ActiveWatcher {
        repo_path: repo_path.clone(),
        stop: Arc::clone(&stop),
    });
    drop(guard);

    let head_path = git_dir.join("HEAD");
    let refs_path = git_dir.join("refs").join("heads");
    let packed_refs_path = git_dir.join("packed-refs");

    thread::spawn(move || {
        let (_channel_guard, rx) = mpsc::sync_channel::<()>(1);
        let event_tx = _channel_guard.clone();
        let mut last_state = read_watch_state(&repo_path);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                let all_lock = event
                    .paths
                    .iter()
                    .all(|p| p.extension().map(|e| e == "lock").unwrap_or(false));
                if !event.paths.is_empty() && all_lock {
                    return;
                }
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                        let _ = event_tx.try_send(());
                    }
                    _ => {}
                }
            }
        })
        .ok();

        if let Some(watcher) = watcher.as_mut() {
            if head_path.exists() {
                let _ = watcher.watch(&head_path, RecursiveMode::NonRecursive);
            }
            if refs_path.is_dir() {
                let _ = watcher.watch(&refs_path, RecursiveMode::Recursive);
            }
            if packed_refs_path.exists() {
                let _ = watcher.watch(&packed_refs_path, RecursiveMode::NonRecursive);
            }
        }

        loop {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            match rx.recv_timeout(Duration::from_millis(WORKTREE_POLL_MS)) {
                Ok(_) => {
                    thread::sleep(Duration::from_millis(DEBOUNCE_MS));
                    while rx.try_recv().is_ok() {}
                    if !stop.load(Ordering::Relaxed) {
                        update_watch_state(&mut last_state, read_watch_state(&repo_path));
                        let _ = app.emit("git-branch-changed", ());
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if update_watch_state(&mut last_state, read_watch_state(&repo_path)) {
                        let _ = app.emit("git-branch-changed", ());
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    });

    Ok(())
}

pub fn stop_git_watcher(repo_path: &Path) -> Result<(), String> {
    let mut guard = WATCHER_ACTIVE
        .lock()
        .map_err(|_| "Lock error".to_string())?;
    stop_matching_watcher(&mut guard, repo_path);
    Ok(())
}

pub(super) fn stop_matching_watcher(slot: &mut Option<ActiveWatcher>, repo_path: &Path) {
    if slot
        .as_ref()
        .is_some_and(|watcher| watcher.repo_path == repo_path)
    {
        if let Some(watcher) = slot.take() {
            watcher.stop.store(true, Ordering::Relaxed);
        }
    }
}
