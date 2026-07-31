use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use super::types_tools::ToolFileChangeStatus;
use super::tool_bash_change_event::PreparedEvent;

const MAX_WORKSPACE_WATCHERS: usize = super::tool_bash_watch_roots::MAX_WATCH_ROOTS;
const MAX_BUFFERED_EVENTS: usize = 4_096;
const SKIPPED_DIRECTORIES: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
];

static HUBS: LazyLock<Mutex<VecDeque<Arc<WorkspaceEventHub>>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));
static CREATE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub struct WorkspaceEventHub {
    root: PathBuf,
    events: Mutex<EventRing>,
}

#[derive(Clone)]
pub struct RecordedEvent {
    pub sequence: u64,
    pub path: PathBuf,
    pub status: ToolFileChangeStatus,
}

#[derive(Default)]
struct EventRing {
    next_sequence: u64,
    last_overflow_sequence: u64,
    events: VecDeque<RecordedEvent>,
}

impl WorkspaceEventHub {
    fn create(root: PathBuf) -> Result<Arc<Self>, String> {
        super::tool_bash_watch_roots::attach(&root)?;
        Ok(Arc::new(Self {
            root,
            events: Mutex::new(EventRing::default()),
        }))
    }

    pub fn sequence(&self) -> u64 {
        self.lock_events().next_sequence
    }

    pub fn events_after(&self, sequence: u64) -> (Vec<RecordedEvent>, bool) {
        let ring = self.lock_events();
        let oldest = ring
            .events
            .front()
            .map_or(ring.next_sequence, |event| event.sequence);
        let gap = sequence.saturating_add(1) < oldest || sequence < ring.last_overflow_sequence;
        let events = ring
            .events
            .iter()
            .filter(|event| event.sequence > sequence)
            .cloned()
            .collect();
        (events, gap)
    }

    fn record(&self, event: &PreparedEvent) {
        let created_directory = event
            .paths
            .iter()
            .any(|entry| entry.is_directory && is_trackable(&self.root, &entry.path));
        let mut paths = event
            .paths
            .iter()
            .filter(|entry| is_trackable(&self.root, &entry.path));
        let Some(first) = paths.next() else {
            if event.truncated {
                self.mark_overflow();
            }
            return;
        };
        let mut ring = self.lock_events();
        ring.push(first.path.clone(), event.status);
        for entry in paths {
            ring.push(entry.path.clone(), event.status);
        }
        if event.truncated || created_directory {
            ring.mark_overflow();
        }
    }

    fn mark_overflow(&self) {
        self.lock_events().mark_overflow();
    }

    fn lock_events(&self) -> std::sync::MutexGuard<'_, EventRing> {
        self.events
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

impl EventRing {
    fn push(&mut self, path: PathBuf, status: ToolFileChangeStatus) {
        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.events.len() >= MAX_BUFFERED_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(RecordedEvent {
            sequence: self.next_sequence,
            path,
            status,
        });
    }

    fn mark_overflow(&mut self) {
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.last_overflow_sequence = self.next_sequence;
    }
}

pub fn workspace_hub(root: PathBuf) -> Result<Arc<WorkspaceEventHub>, String> {
    if let Some(hub) = existing_hub(&root) {
        return Ok(hub);
    }
    let _create_guard = CREATE_LOCK
        .try_lock()
        .map_err(|_| "Initialisation du suivi deja en cours.".to_string())?;
    if let Some(hub) = existing_hub(&root) {
        return Ok(hub);
    }
    evict_inactive_hub()?;
    let created = WorkspaceEventHub::create(root)?;
    lock_hubs().push_back(Arc::clone(&created));
    Ok(created)
}

pub fn handle_notify_event(result: notify::Result<notify::Event>) {
    let hubs = {
        let hubs = lock_hubs();
        hubs.iter().cloned().collect::<Vec<_>>()
    };
    if hubs.is_empty() {
        return;
    }
    match result {
        Ok(event) => {
            let Some(event) = PreparedEvent::capture(event) else {
                return;
            };
            for hub in &hubs {
                hub.record(&event);
            }
        }
        Err(_) => {
            for hub in &hubs {
                hub.mark_overflow();
            }
        }
    }
}

fn existing_hub(root: &Path) -> Option<Arc<WorkspaceEventHub>> {
    let mut hubs = lock_hubs();
    let position = hubs.iter().position(|hub| hub.root == root)?;
    let hub = hubs.remove(position)?;
    hubs.push_back(Arc::clone(&hub));
    Some(hub)
}

fn evict_inactive_hub() -> Result<(), String> {
    let candidate = {
        let mut hubs = lock_hubs();
        if hubs.len() < MAX_WORKSPACE_WATCHERS {
            return Ok(());
        }
        let position = hubs
            .iter()
            .position(|hub| Arc::strong_count(hub) == 1)
            .ok_or_else(|| "Trop de suivis de fichiers actifs.".to_string())?;
        hubs
            .remove(position)
            .map(|hub| hub.root.clone())
            .ok_or_else(|| "Suivi des fichiers indisponible.".to_string())?
    };
    super::tool_bash_watch_roots::detach(&candidate)?;
    Ok(())
}

fn lock_hubs() -> std::sync::MutexGuard<'static, VecDeque<Arc<WorkspaceEventHub>>> {
    HUBS.lock().unwrap_or_else(|error| error.into_inner())
}

pub(super) fn is_trackable(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    !relative.components().any(|component| {
        let std::path::Component::Normal(name) = component else {
            return false;
        };
        SKIPPED_DIRECTORIES
            .iter()
            .any(|skipped| name == std::ffi::OsStr::new(skipped))
    })
}
