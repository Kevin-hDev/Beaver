use super::host_identity::HostIdentity;
use super::ui_catalog::{StoredCatalog, UiCatalog, UiCatalogUpdate};
use super::ui_types::{UiActionPayload, UiCatalogEntry};
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Duration;

fn entry(owner: &str) -> UiCatalogEntry {
    UiCatalogEntry {
        extension_id: owner.to_string(),
        contribution_id: format!("{owner}.toolbar"),
        action_ids: vec![format!("{owner}.run")],
        declared_action_ids: vec![format!("{owner}.run")],
        contribution: json!({"type":"action"}),
    }
}

fn update(identity: HostIdentity, generation: u64, owner: &str) -> UiCatalogUpdate {
    UiCatalogUpdate {
        identity,
        generation,
        extension_id: owner.to_string(),
        entries: vec![entry(owner)],
    }
}

#[test]
fn catalog_revision_is_monotone_and_owner_bound() {
    let catalog = UiCatalog::default();
    let owner = HostIdentity::ThirdParty("com.example.owner".to_string());
    let first = catalog
        .replace(&owner, vec![entry("com.example.owner")])
        .unwrap();
    let second = catalog
        .replace(&owner, vec![entry("com.example.owner")])
        .unwrap();
    assert!(second > first);
    assert!(catalog
        .authorize(
            "com.example.owner",
            "com.example.owner.toolbar",
            "com.example.owner.run"
        )
        .is_ok());
    assert!(catalog
        .authorize(
            "com.example.other",
            "com.example.owner.toolbar",
            "com.example.owner.run"
        )
        .is_err());
    let snapshot = catalog.snapshot().unwrap();
    assert_eq!(snapshot.revision, second);
    assert_eq!(snapshot.contributions.len(), 1);
    let public = serde_json::to_value(snapshot).unwrap();
    assert!(public.pointer("/contributions/0/actionIds").is_none());
    assert!(public
        .pointer("/contributions/0/declaredActionIds")
        .is_none());
    assert!(public.pointer("/contributions/0/catalogRevision").is_none());
    assert!(public.pointer("/contributions/0/identity").is_none());
    assert!(public.pointer("/contributions/0/generation").is_none());
}

#[test]
fn retired_generations_cannot_republish_and_tombstones_stay_bounded() {
    let catalog = UiCatalog::default();
    let owner = HostIdentity::ThirdParty("com.example.owner".to_string());
    catalog
        .apply(vec![update(owner.clone(), 1, "com.example.owner")])
        .unwrap();
    catalog.retire(&owner, 1).unwrap();
    assert!(catalog
        .apply(vec![update(owner.clone(), 1, "com.example.owner")])
        .is_err());
    catalog
        .apply(vec![update(owner, 2, "com.example.owner")])
        .unwrap();

    for index in 0..(super::types::MAX_HOST_PROCESSES * 2) {
        let identity = HostIdentity::ThirdParty(format!("com.example.retired-{index}"));
        catalog.retire(&identity, (index + 3) as u64).unwrap();
    }
    assert!(catalog.state.read().unwrap().retired.len() <= super::types::MAX_HOST_PROCESSES);
}

#[test]
fn official_shared_host_keeps_extensions_distinct_and_spoofing_is_rejected() {
    let catalog = UiCatalog::default();
    catalog
        .apply(vec![
            update(HostIdentity::Official, 7, "beaver.documents"),
            update(HostIdentity::Official, 7, "beaver.pdf"),
        ])
        .unwrap();
    assert_eq!(catalog.snapshot().unwrap().contributions.len(), 2);
    assert!(catalog
        .apply(vec![
            update(HostIdentity::Official, 7, "com.example.spoof",)
        ])
        .is_err());
    assert!(catalog
        .apply(vec![update(
            HostIdentity::ThirdParty("com.example.owner".to_string()),
            8,
            "com.example.other",
        )])
        .is_err());
}

#[test]
fn revision_overflow_fails_before_mutating_the_catalog() {
    let catalog = UiCatalog::default();
    catalog.state.write().unwrap().revision = u64::MAX;
    assert!(catalog
        .apply(vec![update(
            HostIdentity::ThirdParty("com.example.owner".to_string()),
            1,
            "com.example.owner",
        )])
        .is_err());
    assert!(catalog.snapshot().unwrap().contributions.is_empty());
}

#[test]
fn global_contribution_placement_and_serialized_limits_fail_only_the_new_owner() {
    let catalog = UiCatalog::default();
    let mut updates = Vec::new();
    for extension in 0..16 {
        let owner = format!("com.example.global-{extension}");
        let entries = (0..super::ui_contract::MAX_CONTRIBUTIONS_PER_EXTENSION)
            .map(|index| UiCatalogEntry {
                extension_id: owner.clone(),
                contribution_id: format!("{owner}.entry-{index}"),
                action_ids: Vec::new(),
                declared_action_ids: Vec::new(),
                contribution: json!({"type":"theme"}),
            })
            .collect();
        updates.push(UiCatalogUpdate {
            identity: HostIdentity::ThirdParty(owner.clone()),
            generation: (extension + 1) as u64,
            extension_id: owner,
            entries,
        });
    }
    let accepted = catalog.apply(updates).unwrap();
    assert!(accepted.rejected_extensions.is_empty());
    assert_eq!(catalog.snapshot().unwrap().contributions.len(), 512);
    let overflow = catalog
        .apply(vec![update(
            HostIdentity::ThirdParty("com.example.overflow".to_string()),
            17,
            "com.example.overflow",
        )])
        .unwrap();
    assert!(overflow
        .rejected_extensions
        .contains("com.example.overflow"));
    assert_eq!(catalog.snapshot().unwrap().contributions.len(), 512);

    let mut placement_extensions = BTreeMap::new();
    for extension in 0..4 {
        let owner = format!("com.example.placement-{extension}");
        let entries = (0..super::ui_contract::MAX_CONTRIBUTIONS_PER_EXTENSION)
            .map(|index| UiCatalogEntry {
                extension_id: owner.clone(),
                contribution_id: format!("{owner}.entry-{index}"),
                action_ids: Vec::new(),
                declared_action_ids: Vec::new(),
                contribution: json!({"placement":"app.toolbar.primary"}),
            })
            .collect();
        placement_extensions.insert(
            owner.clone(),
            StoredCatalog {
                identity: HostIdentity::ThirdParty(owner),
                generation: (extension + 1) as u64,
                catalog_revision: 1,
                entries,
            },
        );
    }
    assert!(super::ui_catalog_limits::validate(&placement_extensions, 1).is_ok());
    placement_extensions.insert(
        "com.example.placement-overflow".to_string(),
        StoredCatalog {
            identity: HostIdentity::ThirdParty("com.example.placement-overflow".to_string()),
            generation: 5,
            catalog_revision: 1,
            entries: vec![UiCatalogEntry {
                extension_id: "com.example.placement-overflow".to_string(),
                contribution_id: "com.example.placement-overflow.entry".to_string(),
                action_ids: Vec::new(),
                declared_action_ids: Vec::new(),
                contribution: json!({"placement":"app.toolbar.primary"}),
            }],
        },
    );
    assert!(super::ui_catalog_limits::validate(&placement_extensions, 1).is_err());

    let owner = "com.example.sized";
    let mut extensions = BTreeMap::new();
    let mut sized = entry(owner);
    sized.action_ids.clear();
    sized.declared_action_ids.clear();
    sized.contribution = json!({"padding":""});
    extensions.insert(
        owner.to_string(),
        StoredCatalog {
            identity: HostIdentity::ThirdParty(owner.to_string()),
            generation: 1,
            catalog_revision: 1,
            entries: vec![sized.clone()],
        },
    );
    let base = serde_json::to_vec(&super::ui_catalog_limits::snapshot(1, &extensions))
        .unwrap()
        .len();
    sized.contribution = json!({
        "padding":"x".repeat(super::ui_contract::MAX_GLOBAL_UI_BYTES - base)
    });
    extensions.get_mut(owner).unwrap().entries = vec![sized.clone()];
    assert_eq!(
        serde_json::to_vec(&super::ui_catalog_limits::snapshot(1, &extensions))
            .unwrap()
            .len(),
        super::ui_contract::MAX_GLOBAL_UI_BYTES,
    );
    assert!(super::ui_catalog_limits::validate(&extensions, 1).is_ok());
    sized.contribution["padding"] = json!(format!(
        "{}x",
        sized.contribution["padding"].as_str().unwrap()
    ));
    extensions.get_mut(owner).unwrap().entries = vec![sized];
    assert!(super::ui_catalog_limits::validate(&extensions, 1).is_err());
}

#[test]
fn dynamic_view_actions_replace_previous_grants_for_the_same_owner() {
    let catalog = UiCatalog::default();
    let owner = HostIdentity::ThirdParty("com.example.owner".to_string());
    catalog
        .replace(&owner, vec![entry("com.example.owner")])
        .unwrap();
    catalog
        .refresh_actions(
            "com.example.owner",
            "com.example.owner.toolbar",
            1,
            1,
            vec!["com.example.owner.confirm".to_string()],
        )
        .unwrap();
    assert!(catalog
        .authorize(
            "com.example.owner",
            "com.example.owner.toolbar",
            "com.example.owner.confirm",
        )
        .is_ok());
    catalog
        .refresh_actions(
            "com.example.owner",
            "com.example.owner.toolbar",
            1,
            1,
            Vec::new(),
        )
        .unwrap();
    assert!(catalog
        .authorize(
            "com.example.owner",
            "com.example.owner.toolbar",
            "com.example.owner.confirm",
        )
        .is_err());
    assert!(catalog
        .authorize(
            "com.example.owner",
            "com.example.owner.toolbar",
            "com.example.owner.run",
        )
        .is_ok());
}

#[test]
fn dynamic_view_cannot_claim_another_contributions_action() {
    let catalog = UiCatalog::default();
    let owner = HostIdentity::ThirdParty("com.example.owner".to_string());
    let first = entry("com.example.owner");
    let mut second = entry("com.example.owner");
    second.contribution_id = "com.example.owner.secondary".to_string();
    second.action_ids = vec!["com.example.owner.secondary-run".to_string()];
    second.declared_action_ids = second.action_ids.clone();
    catalog.replace(&owner, vec![first, second]).unwrap();
    assert!(catalog
        .refresh_actions(
            "com.example.owner",
            "com.example.owner.toolbar",
            1,
            1,
            vec!["com.example.owner.secondary-run".to_string()],
        )
        .is_err());
}

#[test]
fn dynamic_actions_respect_the_extension_wide_limit() {
    let catalog = UiCatalog::default();
    let owner = HostIdentity::ThirdParty("com.example.owner".to_string());
    let mut first = entry("com.example.owner");
    first.action_ids = (0..32)
        .map(|index| format!("com.example.owner.first-{index}"))
        .collect();
    first.declared_action_ids = first.action_ids.clone();
    let mut second = entry("com.example.owner");
    second.contribution_id = "com.example.owner.secondary".to_string();
    second.action_ids = (0..32)
        .map(|index| format!("com.example.owner.second-{index}"))
        .collect();
    second.declared_action_ids = second.action_ids.clone();
    catalog.replace(&owner, vec![first, second]).unwrap();
    assert!(catalog
        .refresh_actions(
            "com.example.owner",
            "com.example.owner.toolbar",
            1,
            1,
            vec!["com.example.owner.overflow".to_string()],
        )
        .is_err());
}

#[test]
fn an_action_result_cannot_mutate_a_reloaded_catalog_in_the_same_host() {
    let catalog = UiCatalog::default();
    let owner = HostIdentity::ThirdParty("com.example.owner".to_string());
    catalog
        .replace(&owner, vec![entry("com.example.owner")])
        .unwrap();
    let stale = catalog
        .route(
            "com.example.owner",
            "com.example.owner.toolbar",
            "com.example.owner.run",
        )
        .unwrap();
    catalog
        .replace(&owner, vec![entry("com.example.owner")])
        .unwrap();
    assert!(catalog
        .refresh_actions(
            "com.example.owner",
            "com.example.owner.toolbar",
            stale.generation,
            stale.catalog_revision,
            vec!["com.example.owner.late".to_string()],
        )
        .is_err());
}

#[test]
fn action_fields_accept_max_reject_max_plus_one_and_duplicate_keys() {
    let max = super::ui_contract::MAX_FIELDS_PER_VIEW;
    let fields = (0..max)
        .map(|index| format!(r#""field-{index}":{index}"#))
        .collect::<Vec<_>>()
        .join(",");
    let payload: UiActionPayload =
        serde_json::from_str(&format!(r#"{{"fields":{{{fields}}}}}"#)).unwrap();
    assert_eq!(payload.fields.len(), max);
    let overflow = format!(r#"{{"fields":{{{fields},"overflow":true}}}}"#);
    assert!(serde_json::from_str::<UiActionPayload>(&overflow).is_err());
    assert!(
        serde_json::from_str::<UiActionPayload>(r#"{"fields":{"duplicate":1,"duplicate":2}}"#,)
            .is_err()
    );
}

#[tokio::test]
async fn cancellation_wins_over_a_pending_action() {
    let cancellation = tokio_util::sync::CancellationToken::new();
    cancellation.cancel();
    let result = super::ui_dispatch::await_action(
        std::future::pending::<Result<serde_json::Value, String>>(),
        cancellation,
        Duration::from_secs(1),
    )
    .await;
    assert!(result.is_err());
}

#[test]
fn action_results_are_revalidated_and_bounded() {
    assert!(super::ui_action_result::validate(
        "com.example.owner",
        json!({
            "type":"notification", "level":"info", "message":{"default":"Done"}
        })
    )
    .is_ok());
    assert!(super::ui_action_result::validate(
        "com.example.owner",
        json!({
            "type":"notification", "level":"info",
            "message":{"default":"x".repeat(super::ui_contract::MAX_ACTION_RESULT_BYTES)}
        })
    )
    .is_err());
    let view = super::ui_action_result::validate(
        "com.example.owner",
        json!({
            "type":"view", "view":{
                "type":"button", "id":"com.example.owner.confirm",
                "label":{"default":"Confirm"}, "actionId":"com.example.owner.confirm"
            }
        }),
    )
    .unwrap();
    assert_eq!(view.action_ids, ["com.example.owner.confirm"]);
}
