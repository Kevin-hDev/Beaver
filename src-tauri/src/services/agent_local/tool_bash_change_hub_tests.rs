use notify::{event::Flag, event::ModifyKind, Event, EventKind};

use super::{EventRing, WorkspaceEventHub};
use crate::services::agent_local::tool_bash_change_event::PreparedEvent;

fn test_hub(root: &std::path::Path) -> WorkspaceEventHub {
    WorkspaceEventHub {
        root: root.to_path_buf(),
        events: std::sync::Mutex::new(EventRing::default()),
    }
}

#[test]
fn rescan_events_mark_the_hub_as_overflowed() {
    let hub = test_hub(std::path::Path::new("/workspace"));
    let event = Event::new(EventKind::Other).set_flag(Flag::Rescan);
    let prepared = PreparedEvent::capture(event).expect("rescan event");

    hub.record(&prepared);

    assert!(hub.events_after(0).1);
}

#[test]
fn path_limit_applies_after_ignored_paths_are_filtered() {
    let root = std::path::Path::new("/workspace");
    let hub = test_hub(root);
    let mut event = Event::new(EventKind::Modify(ModifyKind::Any));
    event.paths = (0..150)
        .map(|index| root.join(format!("target/ignored-{index}")))
        .chain([root.join("src/kept.rs")])
        .collect();
    let prepared = PreparedEvent::capture(event).expect("change event");

    hub.record(&prepared);

    let (events, overflowed) = hub.events_after(0);
    assert!(!overflowed);
    assert_eq!(events.len(), 1);
    assert!(events[0].path.ends_with("src/kept.rs"));
}

#[test]
fn generated_ecosystem_directories_are_not_tracked() {
    let root = std::path::Path::new("/workspace");
    for directory in [
        ".git",
        "node_modules",
        "target",
        "dist",
        "build",
        ".next",
        ".nuxt",
        ".turbo",
        ".cache",
        ".venv",
        "venv",
        "__pycache__",
        "vendor",
        "Pods",
        ".gradle",
        "out",
        "coverage",
    ] {
        assert!(!super::is_trackable(
            root,
            &root.join(directory).join("generated.file")
        ));
    }
    assert!(super::is_trackable(root, &root.join("src/main.py")));
}
