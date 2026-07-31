use notify::{Event, EventKind};
use std::path::PathBuf;
use std::sync::OnceLock;

use super::types_tools::ToolFileChangeStatus;

const MAX_EVENT_PATHS: usize = 4_096;
pub(super) const MAX_RECORDED_PATHS: usize = 128;

pub(super) struct PreparedEvent {
    pub paths: Vec<PreparedPath>,
    pub status: Option<ToolFileChangeStatus>,
    pub rescan: bool,
    pub input_truncated: bool,
}

pub(super) struct PreparedPath {
    pub path: PathBuf,
    check_directory: bool,
    is_directory: OnceLock<bool>,
}

impl PreparedPath {
    pub fn is_directory(&self) -> bool {
        self.check_directory && *self.is_directory.get_or_init(|| self.path.is_dir())
    }
}

impl PreparedEvent {
    pub fn capture(mut event: Event) -> Option<Self> {
        let status = status_for_kind(&event.kind);
        let rescan = event.need_rescan();
        if status.is_none() && !rescan {
            return None;
        }
        let check_directories =
            cfg!(target_os = "linux") && matches!(event.kind, EventKind::Create(_));
        let input_truncated = event.paths.len() > MAX_EVENT_PATHS;
        event.paths.truncate(MAX_EVENT_PATHS);
        let paths = event
            .paths
            .into_iter()
            .map(|path| PreparedPath {
                path,
                check_directory: check_directories,
                is_directory: OnceLock::new(),
            })
            .collect();
        Some(Self {
            paths,
            status,
            rescan,
            input_truncated,
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
    fn bounds_raw_paths_before_dispatching_to_hubs() {
        let mut event = Event::new(EventKind::Create(CreateKind::Any));
        event.paths = (0..4_200).map(|index| index.to_string().into()).collect();

        let prepared = super::PreparedEvent::capture(event).expect("prepared event");

        assert_eq!(prepared.paths.len(), super::MAX_EVENT_PATHS);
        assert!(prepared.input_truncated);
        assert!(prepared
            .paths
            .iter()
            .all(|path| path.is_directory.get().is_none()));
    }
}
