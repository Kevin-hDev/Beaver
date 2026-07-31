use notify::{Event, EventKind};
use std::path::PathBuf;

use super::types_tools::ToolFileChangeStatus;

const MAX_PATHS_PER_EVENT: usize = 128;

pub(super) struct PreparedEvent {
    pub paths: Vec<PreparedPath>,
    pub status: ToolFileChangeStatus,
    pub truncated: bool,
}

pub(super) struct PreparedPath {
    pub path: PathBuf,
    pub is_directory: bool,
}

impl PreparedEvent {
    pub fn capture(event: Event) -> Option<Self> {
        let status = status_for_kind(&event.kind)?;
        let check_directories =
            cfg!(target_os = "linux") && matches!(event.kind, EventKind::Create(_));
        let paths = event
            .paths
            .iter()
            .take(MAX_PATHS_PER_EVENT)
            .map(|path| PreparedPath {
                path: path.clone(),
                is_directory: check_directories && path.is_dir(),
            })
            .collect();
        Some(Self {
            paths,
            status,
            truncated: event.paths.len() > MAX_PATHS_PER_EVENT,
        })
    }
}

fn status_for_kind(kind: &EventKind) -> Option<ToolFileChangeStatus> {
    match kind {
        EventKind::Create(_) => Some(ToolFileChangeStatus::Added),
        EventKind::Modify(_) => Some(ToolFileChangeStatus::Modified),
        EventKind::Remove(_) => Some(ToolFileChangeStatus::Deleted),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use notify::{event::CreateKind, Event, EventKind};

    #[test]
    fn bounds_paths_before_dispatching_to_hubs() {
        let mut event = Event::new(EventKind::Create(CreateKind::Any));
        event.paths = (0..150).map(|index| index.to_string().into()).collect();

        let prepared = super::PreparedEvent::capture(event).expect("prepared event");

        assert_eq!(prepared.paths.len(), super::MAX_PATHS_PER_EVENT);
        assert!(prepared.truncated);
    }
}
