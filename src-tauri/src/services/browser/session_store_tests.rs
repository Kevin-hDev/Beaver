use super::{
    session_model::SessionModel,
    session_store::{self, E2eSessionKeyAction, SESSION_KEY_BYTES},
    session_types::BrowserRuntimeTabUpdate,
};
use base64::Engine;
use std::fs;
use zeroize::Zeroizing;

fn key() -> Zeroizing<Vec<u8>> {
    Zeroizing::new(vec![7_u8; SESSION_KEY_BYTES])
}

fn id(index: usize) -> String {
    format!("{index:032x}")
}

fn e2e_key(byte: u8) -> zeroize::Zeroizing<String> {
    zeroize::Zeroizing::new(
        base64::engine::general_purpose::STANDARD.encode([byte; SESSION_KEY_BYTES]),
    )
}

#[test]
fn e2e_fixture_seeds_before_random_generation_and_keeps_the_same_second_key() {
    let key = e2e_key(7);

    assert!(matches!(
        session_store::e2e_session_key_action(None, &key),
        Ok(E2eSessionKeyAction::Seed)
    ));
    assert!(matches!(
        session_store::e2e_session_key_action(Some(&key), &key),
        Ok(E2eSessionKeyAction::Keep)
    ));
}

#[test]
fn e2e_fixture_rejects_invalid_or_replaced_keys_without_a_seed_action() {
    let key = e2e_key(7);
    let replacement = e2e_key(8);
    let malformed = format!("{}%", &key[..43]);

    assert!(session_store::e2e_session_key_action(None, "").is_err());
    assert!(session_store::e2e_session_key_action(None, &key[..43]).is_err());
    assert!(session_store::e2e_session_key_action(None, &malformed).is_err());
    assert!(session_store::e2e_session_key_action(Some(&key), &replacement).is_err());
}

#[test]
fn encrypted_round_trip_never_exposes_the_url() {
    let temp = tempfile::tempdir().unwrap();
    let mut model = SessionModel::new("00000000000000000000000000000001".into()).unwrap();
    model
        .navigate(
            "00000000000000000000000000000001",
            "https://example.com/private-path",
        )
        .unwrap();

    session_store::save_at(
        temp.path(),
        "550e8400-e29b-41d4-a716-446655440000",
        &key(),
        &model,
    )
    .unwrap();
    let path = temp.path().join("550e8400-e29b-41d4-a716-446655440000.enc");
    let raw = fs::read(&path).unwrap();
    assert!(!String::from_utf8_lossy(&raw).contains("private-path"));

    let restored =
        session_store::load_at(temp.path(), "550e8400-e29b-41d4-a716-446655440000", &key())
            .unwrap()
            .unwrap();
    assert_eq!(restored.state().tabs[0].url, model.state().tabs[0].url);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn tampering_fails_closed() {
    let temp = tempfile::tempdir().unwrap();
    let model = SessionModel::new("00000000000000000000000000000001".into()).unwrap();
    let session_id = "550e8400-e29b-41d4-a716-446655440000";
    session_store::save_at(temp.path(), session_id, &key(), &model).unwrap();
    let path = temp.path().join(format!("{session_id}.enc"));
    let mut raw = fs::read(&path).unwrap();
    let last = raw.len() - 2;
    raw[last] ^= 1;
    fs::write(&path, raw).unwrap();

    assert!(session_store::load_at(temp.path(), session_id, &key()).is_err());
}

#[test]
fn rejects_invalid_identifiers_and_oversized_files() {
    let temp = tempfile::tempdir().unwrap();
    assert!(session_store::load_at(temp.path(), "../escape", &key()).is_err());
    let session_id = "550e8400-e29b-41d4-a716-446655440000";
    fs::write(
        temp.path().join(format!("{session_id}.enc")),
        vec![0_u8; session_store::MAX_SESSION_FILE_BYTES + 1],
    )
    .unwrap();
    assert!(session_store::load_at(temp.path(), session_id, &key()).is_err());
}

#[test]
fn reordered_session_round_trips_with_its_state_and_recency() {
    let temp = tempfile::tempdir().unwrap();
    let session_id = "550e8400-e29b-41d4-a716-446655440000";
    let mut model = SessionModel::new(id(1)).unwrap();
    model.create_tab(id(2), None).unwrap();
    model
        .update_runtime(
            &id(1),
            &BrowserRuntimeTabUpdate {
                title: Some("Premier".into()),
                ..Default::default()
            },
        )
        .unwrap();
    model
        .update_runtime(
            &id(2),
            &BrowserRuntimeTabUpdate {
                title: Some("Deuxième".into()),
                ..Default::default()
            },
        )
        .unwrap();
    model.navigate(&id(1), "https://example.com/a").unwrap();
    let before = model.persisted();
    model.reorder_tabs(&[id(2), id(1)]).unwrap();

    session_store::save_at(temp.path(), session_id, &key(), &model).unwrap();
    let restored = session_store::load_at(temp.path(), session_id, &key())
        .unwrap()
        .unwrap()
        .persisted();

    assert_eq!(restored.state.tabs[0], before.state.tabs[1]);
    assert_eq!(restored.state.tabs[1], before.state.tabs[0]);
    assert_eq!(restored.state.active_tab_id, before.state.active_tab_id);
    assert_eq!(restored.recency, before.recency);
}

#[test]
fn rejected_orders_do_not_mutate_a_regular_or_saturated_session() {
    let mut model = SessionModel::new(id(1)).unwrap();
    model.create_tab(id(2), None).unwrap();
    let before = model.persisted();

    assert!(model.reorder_tabs(&[id(1), id(1)]).is_err());
    assert_eq!(model.persisted().state, before.state);
    assert_eq!(model.persisted().recency, before.recency);

    let mut value = serde_json::to_value(model.persisted()).unwrap();
    value["state"]["generation"] = serde_json::json!(u64::MAX);
    let mut saturated = SessionModel::restore(&serde_json::to_vec(&value).unwrap()).unwrap();
    let saturated_before = saturated.persisted();

    assert!(saturated.reorder_tabs(&[id(2), id(1)]).is_err());
    assert_eq!(saturated.persisted().state, saturated_before.state);
    assert_eq!(saturated.persisted().recency, saturated_before.recency);
    saturated.reorder_tabs(&[id(1), id(2)]).unwrap();
    assert_eq!(saturated.persisted().state, saturated_before.state);
}
