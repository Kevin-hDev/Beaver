use super::{build_update, enabled, valid_remote_value};
use crate::services::forecast::dev_update_sources::{self, SourceSpec};
use std::collections::HashSet;

#[test]
fn release_builds_never_enable_forecast_checks() {
    assert!(!enabled(false));
    assert!(enabled(true));
}

#[test]
fn sources_are_bounded_unique_and_official() {
    let sources = dev_update_sources::all().unwrap();
    assert!(sources.len() <= dev_update_sources::MAX_SOURCES);
    let mut ids = HashSet::with_capacity(dev_update_sources::MAX_SOURCES);
    for source in sources {
        let id = match source {
            SourceSpec::Pypi { id, .. }
            | SourceSpec::Github { id, .. }
            | SourceSpec::HuggingFace { id, .. } => id,
        };
        assert!(ids.insert(id));
    }
}

#[test]
fn malformed_versions_and_commits_are_rejected() {
    assert!(valid_remote_value("2.3.1", false));
    assert!(!valid_remote_value("2.3.1/latest", false));
    assert!(valid_remote_value(&"a".repeat(40), true));
    assert!(!valid_remote_value(&"a".repeat(39), true));
}

#[test]
fn unchanged_version_creates_no_notification() {
    let source = SourceSpec::Pypi {
        id: "chronos",
        name: "Chronos",
        family: "chronos-bolt",
        package: "chronos-forecasting",
    };
    assert!(build_update(source, "2.3.1").is_none());
    let update = build_update(source, "2.4.0").unwrap();
    assert_eq!(update.current, "2.3.1");
    assert_eq!(update.latest, "2.4.0");
}

#[test]
fn github_updates_require_a_full_commit_hash() {
    let source = SourceSpec::Github {
        id: "kairos-engine",
        name: "Kairos",
        repository: "foundation-model-research/Kairos",
        current: "0322393840ccf6e2bfe9c663f9dcd088a5a7ee07",
    };
    assert!(build_update(source, "main").is_none());
    assert!(build_update(source, &"b".repeat(40)).is_some());
}
