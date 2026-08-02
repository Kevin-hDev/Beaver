use super::{decode, ToolMetricStore, STORE_VERSION};
use crate::services::agent_local::tool_metrics::{ToolMetricEntry, MAX_TRACKED_TOOLS};

#[test]
fn decoder_rejects_oversized_or_invalid_external_collections() {
    let too_many = ToolMetricStore {
        version: STORE_VERSION,
        entries: (0..=MAX_TRACKED_TOOLS)
            .map(|index| ToolMetricEntry {
                name: format!("extension.tool{index}"),
                ..Default::default()
            })
            .collect(),
    };
    assert!(decode(&serde_json::to_vec(&too_many).unwrap()).is_err());

    let invalid = ToolMetricStore {
        version: STORE_VERSION,
        entries: vec![ToolMetricEntry {
            name: "../escape".into(),
            ..Default::default()
        }],
    };
    assert!(decode(&serde_json::to_vec(&invalid).unwrap()).is_err());

    let duplicate = ToolMetricStore {
        version: STORE_VERSION,
        entries: vec![valid_entry("bash"), valid_entry("bash")],
    };
    assert!(decode(&serde_json::to_vec(&duplicate).unwrap()).is_err());
}

fn valid_entry(name: &str) -> ToolMetricEntry {
    ToolMetricEntry {
        name: name.into(),
        invocations: 1,
        success: 1,
        updated_at: 1,
        ..Default::default()
    }
}

#[test]
fn decoder_rejects_inconsistent_external_counters() {
    let store = ToolMetricStore {
        version: STORE_VERSION,
        entries: vec![ToolMetricEntry {
            name: "bash".into(),
            invocations: 1,
            success: 1,
            failed: 1,
            updated_at: 1,
            ..Default::default()
        }],
    };

    assert!(decode(&serde_json::to_vec(&store).unwrap()).is_err());
}

#[test]
fn decoder_rejects_unknown_versions() {
    let store = ToolMetricStore {
        version: STORE_VERSION + 1,
        entries: Vec::new(),
    };

    assert!(decode(&serde_json::to_vec(&store).unwrap()).is_err());
}
