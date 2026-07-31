use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};

use super::types_tools::ToolFileChangeStatus;

const MAX_WORKSPACE_WATCHERS: usize = 16;
const MAX_BUFFERED_EVENTS: usize = 4_096;
const MAX_PATHS_PER_EVENT: usize = 128;
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

pub struct WorkspaceEventHub {
    root: PathBuf,
    events: Arc<Mutex<EventRing>>,
    _watcher: RecommendedWatcher,
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
        let events = Arc::new(Mutex::new(EventRing::default()));
        let callback_events = Arc::clone(&events);
        let callback_root = root.clone();
        let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
            let Ok(event) = result else { return };
            let Some(status) = status_for_kind(&event.kind) else {
                return;
            };
            let mut paths = event
                .paths
                .into_iter()
                .filter(|path| is_trackable(&callback_root, path));
            let Some(first_path) = paths.next() else {
                return;
            };
            let mut ring = callback_events
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            ring.push(first_path, status);
            for path in paths.by_ref().take(MAX_PATHS_PER_EVENT - 1) {
                ring.push(path, status);
            }
            if paths.next().is_some() {
                ring.mark_overflow();
            }
        })
        .map_err(|_| "Suivi des fichiers indisponible.".to_string())?;
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|_| "Suivi des fichiers indisponible.".to_string())?;
        Ok(Arc::new(Self {
            root,
            events,
            _watcher: watcher,
        }))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn sequence(&self) -> u64 {
        self.events
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .next_sequence
    }

    pub fn events_after(&self, sequence: u64) -> (Vec<RecordedEvent>, bool) {
        let ring = self
            .events
            .lock()
            .unwrap_or_else(|error| error.into_inner());
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
    {
        let mut hubs = HUBS.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(hub) = take_existing(&mut hubs, &root) {
            return Ok(hub);
        }
    }
    let created = WorkspaceEventHub::create(root.clone())?;
    let mut hubs = HUBS.lock().unwrap_or_else(|error| error.into_inner());
    if let Some(hub) = take_existing(&mut hubs, &root) {
        return Ok(hub);
    }
    if hubs.len() >= MAX_WORKSPACE_WATCHERS {
        hubs.pop_front();
    }
    hubs.push_back(Arc::clone(&created));
    Ok(created)
}

fn take_existing(
    hubs: &mut VecDeque<Arc<WorkspaceEventHub>>,
    root: &Path,
) -> Option<Arc<WorkspaceEventHub>> {
    let position = hubs.iter().position(|hub| hub.root == root)?;
    let hub = hubs.remove(position)?;
    hubs.push_back(Arc::clone(&hub));
    Some(hub)
}

fn status_for_kind(kind: &EventKind) -> Option<ToolFileChangeStatus> {
    match kind {
        EventKind::Create(_) => Some(ToolFileChangeStatus::Added),
        EventKind::Modify(_) => Some(ToolFileChangeStatus::Modified),
        EventKind::Remove(_) => Some(ToolFileChangeStatus::Deleted),
        _ => None,
    }
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
