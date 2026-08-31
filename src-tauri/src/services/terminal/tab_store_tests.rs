use super::tab_store::{
    parse_document, save_with, serialize_document, validate_document, TerminalSavedTab,
    TerminalTabsDocument,
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn legacy_cwd_is_read_but_never_republished() {
    let legacy = br#"{"project-a":[{"label":"Serveur","cwd":"/forged"}]}"#;
    let parsed = parse_document(legacy).expect("legacy document");
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.groups["project-a"][0].label, "Serveur");
    let encoded = serialize_document(&parsed).expect("versioned document");
    assert!(!String::from_utf8(encoded).unwrap().contains("cwd"));
}

#[test]
fn rejects_more_than_the_total_tab_limit() {
    let mut document = TerminalTabsDocument::empty();
    for group in 0..17 {
        document.groups.insert(
            format!("group-{group}"),
            (0..16)
                .map(|tab| TerminalSavedTab {
                    label: format!("tab-{tab}"),
                })
                .collect(),
        );
    }
    assert_eq!(
        validate_document(&document),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_a_file_larger_than_one_mib_before_json() {
    let oversized = vec![b' '; 1024 * 1024 + 1];

    assert_eq!(
        parse_document(&oversized),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_an_unknown_version() {
    assert_eq!(
        parse_document(br#"{"version":2,"groups":{}}"#),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_more_than_128_groups() {
    let mut document = TerminalTabsDocument::empty();
    for group in 0..129 {
        document.groups.insert(format!("group-{group}"), Vec::new());
    }

    assert_eq!(
        validate_document(&document),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_more_than_16_tabs_in_one_group() {
    let mut document = TerminalTabsDocument::empty();
    document.groups.insert(
        "project-a".into(),
        (0..17)
            .map(|tab| TerminalSavedTab {
                label: format!("tab-{tab}"),
            })
            .collect(),
    );

    assert_eq!(
        validate_document(&document),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_a_group_key_larger_than_128_bytes() {
    let mut document = TerminalTabsDocument::empty();
    document.groups.insert("g".repeat(129), Vec::new());

    assert_eq!(
        validate_document(&document),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_a_label_larger_than_512_bytes() {
    let document = document_with("project-a", "l".repeat(513));

    assert_eq!(
        validate_document(&document),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_empty_group_keys_and_labels() {
    let mut empty_group = TerminalTabsDocument::empty();
    empty_group.groups.insert(String::new(), Vec::new());
    let empty_label = document_with("project-a", String::new());

    assert_eq!(
        validate_document(&empty_group),
        Err("terminal-tabs-invalid".into())
    );
    assert_eq!(
        validate_document(&empty_label),
        Err("terminal-tabs-invalid".into())
    );
}

#[test]
fn rejects_nul_carriage_return_and_line_feed() {
    for invalid in ["bad\0text", "bad\rtext", "bad\ntext"] {
        let invalid_group = document_with(invalid, "valid");
        let invalid_label = document_with("project-a", invalid);

        assert_eq!(
            validate_document(&invalid_group),
            Err("terminal-tabs-invalid".into())
        );
        assert_eq!(
            validate_document(&invalid_label),
            Err("terminal-tabs-invalid".into())
        );
    }
}

#[test]
fn duplicate_group_keys_have_one_btree_map_authority() {
    let mut document = TerminalTabsDocument::empty();
    document.groups.insert(
        "project-a".into(),
        vec![TerminalSavedTab {
            label: "first".into(),
        }],
    );
    document.groups.insert(
        "project-a".into(),
        vec![TerminalSavedTab {
            label: "second".into(),
        }],
    );

    assert_eq!(document.groups.len(), 1);
    assert_eq!(document.groups["project-a"][0].label, "second");
}

#[test]
fn legacy_array_is_migrated_to_the_default_group() {
    let parsed =
        parse_document(br#"[{"label":"Shell","cwd":"/ignored"}]"#).expect("legacy array document");

    assert_eq!(
        parsed.groups,
        BTreeMap::from([(
            "__default__".into(),
            vec![TerminalSavedTab {
                label: "Shell".into(),
            }],
        )])
    );
}

#[tokio::test]
async fn concurrent_saves_are_serialized() {
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));

    let save_one = instrumented_save(active.clone(), max_active.clone());
    let save_two = instrumented_save(active.clone(), max_active.clone());
    let (first, second) = tokio::join!(save_one, save_two);

    assert_eq!(first, Ok(()));
    assert_eq!(second, Ok(()));
    assert_eq!(max_active.load(Ordering::SeqCst), 1);
}

fn document_with(group: impl Into<String>, label: impl Into<String>) -> TerminalTabsDocument {
    TerminalTabsDocument {
        version: 1,
        groups: BTreeMap::from([(
            group.into(),
            vec![TerminalSavedTab {
                label: label.into(),
            }],
        )]),
    }
}

async fn instrumented_save(
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
) -> Result<(), String> {
    save_with(
        TerminalTabsDocument::empty(),
        move |_path, _bytes| async move {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            max_active.fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(20)).await;
            active.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        },
    )
    .await
}
