use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionKind, ExtensionManifest, ExtensionRecord,
    ExtensionResource, ExtensionResourceType, ExtensionSkill, ExtensionStatus,
};
use crate::services::agent_local::extension_session_state::DiscoveryEpoch;
use crate::services::agent_local::extension_tool_selection::PluginDescriptor;

pub(super) struct TestRegistry {
    pub first: ExtensionRecord,
    pub second: ExtensionRecord,
    _first_root: tempfile::TempDir,
    _second_root: tempfile::TempDir,
}

impl TestRegistry {
    pub async fn new() -> Self {
        let first_root = tempfile::tempdir().expect("first extension root");
        let second_root = tempfile::tempdir().expect("second extension root");
        write_fixture_files(first_root.path(), "first");
        write_fixture_files(second_root.path(), "second");
        let first = record("example.first", first_root.path());
        let second = record("example.second", second_root.path());
        Self {
            first,
            second,
            _first_root: first_root,
            _second_root: second_root,
        }
    }

    pub fn records(&self) -> Vec<ExtensionRecord> {
        vec![self.first.clone(), self.second.clone()]
    }

    pub fn plugins(
        &self,
        records: &[ExtensionRecord],
    ) -> Vec<super::registry_index::IndexedPlugin> {
        super::registry_index::plugins_from_records(records)
    }

    pub async fn session(&self, provider: &str, model: &str) -> String {
        let session = crate::services::agent_local::session_store::create_full(
            "Extension loader test",
            model,
            provider,
            false,
            None,
        )
        .await
        .expect("main session");
        self.configure(&session.id, provider, model).await;
        session.id
    }

    pub async fn configure(&self, session_id: &str, provider: &str, model: &str) {
        crate::services::agent_local::extension_session_state::mutate(session_id, |state| {
            *state = crate::services::agent_local::extension_session_state::ExtensionSessionState {
                epoch: Some(DiscoveryEpoch {
                provider: provider.into(),
                model: model.into(),
                context_window: 128_000,
                    catalog_version: "0".repeat(64),
                masked: false,
                }),
                plugin_tool_capacity: super::types::MAX_TOOLS,
                plugin_descriptors: [&self.first, &self.second]
                    .into_iter()
                    .map(|record| PluginDescriptor {
                        id: record.manifest.id.clone(),
                        tool_count: 0,
                        definition_count: 0,
                    })
                    .collect(),
                active_plugin_ids: vec![
                    self.first.manifest.id.clone(),
                    self.second.manifest.id.clone(),
                ],
                discovered_plugin_ids: Vec::new(),
            };
            Ok(())
        })
        .await
        .expect("configure extension session");
    }

    pub async fn inspect(&self, session_id: &str) {
        crate::services::agent_local::extension_session_state::mutate(session_id, |state| {
            state.discovered_plugin_ids = vec![
                self.first.manifest.id.clone(),
                self.second.manifest.id.clone(),
            ];
            Ok(())
        })
        .await
        .expect("record inspected extensions");
        let state = crate::services::agent_local::extension_session_state::read(session_id)
            .await
            .expect("inspection state");
        assert_eq!(
            state.discovered_plugin_ids,
            ["example.first", "example.second"]
        );
    }
}

fn write_fixture_files(root: &std::path::Path, label: &str) {
    std::fs::write(root.join("reference.txt"), label).expect("reference");
    std::fs::write(root.join("SKILL.md"), format!("# Guide\n\n{label}")).expect("skill");
    std::fs::write(root.join("image.bin"), b"\x89PNG\r\n\x1a\n").expect("image");
    std::fs::write(root.join("unknown.bin"), [0_u8, 1, 2]).expect("binary");
    std::fs::write(
        root.join("text-exact.txt"),
        vec![b'x'; super::types::MAX_TEXT_RESOURCE_BYTES],
    )
    .expect("exact text");
    std::fs::write(
        root.join("text-too-large.bin"),
        vec![b'x'; super::types::MAX_TEXT_RESOURCE_BYTES + 1],
    )
    .expect("oversized text");
}

fn record(id: &str, root: &std::path::Path) -> ExtensionRecord {
    ExtensionRecord {
        manifest: ExtensionManifest {
            id: id.into(),
            name: id.into(),
            version: "1.0.0".into(),
            beaver_api: super::types::BEAVER_API_VERSION.into(),
            runtime: "node".into(),
            main: Some("index.mjs".into()),
            ui: None,
            ui_legacy: None,
            access: "full".into(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: Some("P3 fixture".into()),
        },
        kind: ExtensionKind::Local,
        source: root.to_string_lossy().into_owned(),
        origin: None,
        enabled: true,
        trusted: true,
        fingerprint: None,
        ui_artifact: None,
        trusted_at: None,
        show_in_chat: false,
        status: ExtensionStatus::Active,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions {
            skills: vec![ExtensionSkill {
                id: "guide".into(),
                name: "Guide".into(),
                description: "Guide fixture".into(),
                path: "SKILL.md".into(),
            }],
            resources: vec![
                resource("reference", ExtensionResourceType::Text, "reference.txt"),
                resource("image-as-text", ExtensionResourceType::Text, "image.bin"),
                resource("unknown", ExtensionResourceType::File, "unknown.bin"),
                resource("text-exact", ExtensionResourceType::Text, "text-exact.txt"),
                resource(
                    "text-too-large",
                    ExtensionResourceType::Image,
                    "text-too-large.bin",
                ),
                resource("missing", ExtensionResourceType::File, "missing.bin"),
            ],
            ..Default::default()
        },
    }
}

fn resource(id: &str, kind: ExtensionResourceType, path: &str) -> ExtensionResource {
    ExtensionResource {
        id: id.into(),
        name: id.into(),
        description: "Resource fixture".into(),
        resource_type: kind,
        path: path.into(),
    }
}
