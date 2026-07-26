use super::{
    session_model::{BrowserTabCreation, SessionModel, MAX_BROWSER_TABS},
    session_types::BrowserRuntimeTabUpdate,
};

fn id(index: usize) -> String {
    format!("{index:032x}")
}

#[test]
fn creates_and_closes_tabs_without_ever_becoming_empty() {
    let mut model = SessionModel::new(id(1)).unwrap();
    let created = model.create_tab(id(2), None).unwrap();
    assert!(matches!(created, BrowserTabCreation::Created { .. }));
    assert_eq!(model.state().tabs.len(), 2);

    model.close_tab(&id(2), id(3)).unwrap();
    model.close_tab(&id(1), id(3)).unwrap();

    assert_eq!(model.state().tabs.len(), 1);
    assert_eq!(model.state().active_tab_id, id(3));
}

#[test]
fn asks_before_replacing_the_oldest_inactive_tab() {
    let mut model = SessionModel::new(id(1)).unwrap();
    for index in 2..=MAX_BROWSER_TABS {
        model.create_tab(id(index), None).unwrap();
    }
    model.activate_tab(&id(5)).unwrap();

    let result = model.create_tab(id(11), None).unwrap();
    let BrowserTabCreation::ConfirmationRequired { candidate_id, .. } = result else {
        panic!("confirmation attendue");
    };
    assert_eq!(candidate_id, id(1));
    assert_eq!(model.state().tabs.len(), MAX_BROWSER_TABS);

    let replaced = model.create_tab(id(11), Some(&candidate_id)).unwrap();
    assert!(matches!(replaced, BrowserTabCreation::Created { .. }));
    assert_eq!(model.state().tabs.len(), MAX_BROWSER_TABS);
    assert!(!model.state().tabs.iter().any(|tab| tab.id == candidate_id));
}

#[test]
fn rejects_the_wrong_replacement_and_duplicate_ids() {
    let mut model = SessionModel::new(id(1)).unwrap();
    assert!(model.create_tab(id(1), None).is_err());
    for index in 2..=MAX_BROWSER_TABS {
        model.create_tab(id(index), None).unwrap();
    }
    assert!(model.create_tab(id(12), Some(&id(3))).is_err());
}

#[test]
fn rejects_a_replacement_before_the_tab_limit() {
    let mut model = SessionModel::new(id(1)).unwrap();

    assert!(model.create_tab(id(2), Some(&id(1))).is_err());
    assert_eq!(model.state().tabs.len(), 1);
}

#[test]
fn restores_order_and_marks_pages_as_released() {
    let mut model = SessionModel::new(id(1)).unwrap();
    model.create_tab(id(2), None).unwrap();
    model.navigate(&id(2), "https://example.com/path").unwrap();

    let bytes = serde_json::to_vec(&model.persisted()).unwrap();
    let mut restored = SessionModel::restore(&bytes).unwrap();
    restored.release_runtime().unwrap();

    assert_eq!(restored.state().tabs[0].id, id(1));
    assert_eq!(
        restored.state().tabs[1].url.as_deref(),
        Some("https://example.com/path")
    );
    assert!(restored.state().tabs[1].released);
    assert!(!restored.state().tabs[1].loading);
}

#[test]
fn rejects_unbounded_or_invalid_restored_state() {
    let model = SessionModel::new(id(1)).unwrap();
    let mut value = serde_json::to_value(model.persisted()).unwrap();
    let tabs = value["state"]["tabs"].as_array_mut().unwrap();
    for index in 2..=MAX_BROWSER_TABS + 1 {
        let mut tab = tabs[0].clone();
        tab["id"] = serde_json::Value::String(id(index));
        tabs.push(tab);
    }
    assert!(SessionModel::restore(&serde_json::to_vec(&value).unwrap()).is_err());

    value["state"]["tabs"] = serde_json::json!([]);
    assert!(SessionModel::restore(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn runtime_updates_change_only_the_target_tab_and_bound_its_title() {
    let mut model = SessionModel::new(id(1)).unwrap();
    model.create_tab(id(2), None).unwrap();
    model
        .update_runtime(
            &id(1),
            &BrowserRuntimeTabUpdate {
                title: Some(format!("{}\nignored", "é".repeat(90))),
                url: Some("https://example.com/updated".into()),
                loading: Some(true),
                can_go_back: Some(true),
                can_go_forward: Some(false),
            },
        )
        .unwrap();

    let first = &model.state().tabs[0];
    let second = &model.state().tabs[1];
    assert_eq!(first.title.chars().count(), 80);
    assert_eq!(first.url.as_deref(), Some("https://example.com/updated"));
    assert!(first.loading && first.can_go_back && !first.can_go_forward);
    assert!(second.title.is_empty() && second.url.is_none());

    model.mark_released(&id(1)).unwrap();
    assert!(model.state().tabs[0].released);
    assert!(!model.state().tabs[0].loading);
}
