use super::store::{UpdateProgressStore, MAX_OPERATIONS};
use super::types::{
    UpdateOperationKind, UpdateOperationPhase, UpdateOperationSnapshot, UpdateOperationStatus,
    UpdateProgressMode,
};

fn operation(id: &str, sequence: u64, status: UpdateOperationStatus) -> UpdateOperationSnapshot {
    UpdateOperationSnapshot {
        id: id.into(),
        sequence,
        kind: UpdateOperationKind::AppRelease,
        label: format!("Update {id}"),
        status,
        phase: UpdateOperationPhase::Downloading,
        progress_mode: UpdateProgressMode::Determinate,
        percent: Some(25),
        queue_position: None,
        can_cancel: !status.is_terminal(),
        can_retry: status == UpdateOperationStatus::Failed,
        is_update: None,
        error_key: None,
        missing_bytes: None,
    }
}

#[test]
fn keeps_four_operations_and_replays_the_snapshot() {
    let mut store = UpdateProgressStore::default();
    for index in 0..4 {
        assert!(store
            .upsert(operation(
                &format!("id-{index}"),
                1,
                UpdateOperationStatus::Running
            ))
            .unwrap());
    }
    assert_eq!(store.snapshot().len(), 4);
    assert_eq!(store.snapshot()[0].id, "id-0");
}

#[test]
fn evicts_the_oldest_terminal_but_never_a_living_operation() {
    let mut store = UpdateProgressStore::default();
    store
        .upsert(operation("old", 1, UpdateOperationStatus::Completed))
        .unwrap();
    for index in 1..MAX_OPERATIONS {
        store
            .upsert(operation(
                &format!("live-{index}"),
                1,
                UpdateOperationStatus::Running,
            ))
            .unwrap();
    }
    store
        .upsert(operation("replacement", 1, UpdateOperationStatus::Running))
        .unwrap();
    assert!(store.get("old").is_none());
    assert!(store.get("replacement").is_some());

    let mut live = UpdateProgressStore::default();
    for index in 0..MAX_OPERATIONS {
        live.upsert(operation(
            &format!("live-{index}"),
            1,
            UpdateOperationStatus::Running,
        ))
        .unwrap();
    }
    assert_eq!(
        live.upsert(operation("too-many", 1, UpdateOperationStatus::Running)),
        Err("update-progress-full")
    );
}

#[test]
fn ignores_stale_sequences_and_bounds_external_fields() {
    let mut store = UpdateProgressStore::default();
    store
        .upsert(operation("same", 2, UpdateOperationStatus::Running))
        .unwrap();
    assert!(!store
        .upsert(operation("same", 1, UpdateOperationStatus::Completed))
        .unwrap());
    assert_eq!(store.get("same").unwrap().sequence, 2);

    let mut invalid_percent = operation("percent", 1, UpdateOperationStatus::Running);
    invalid_percent.percent = Some(101);
    assert_eq!(
        store.upsert(invalid_percent),
        Err("update-progress-invalid")
    );

    let mut long_label = operation("label", 1, UpdateOperationStatus::Running);
    long_label.label = "é".repeat(201);
    assert_eq!(store.upsert(long_label), Err("update-progress-invalid"));

    let mut false_progress = operation("mode", 1, UpdateOperationStatus::Running);
    false_progress.progress_mode = UpdateProgressMode::Indeterminate;
    assert_eq!(store.upsert(false_progress), Err("update-progress-invalid"));

    let mut false_retry = operation("retry", 1, UpdateOperationStatus::Running);
    false_retry.can_retry = true;
    assert_eq!(store.upsert(false_retry), Err("update-progress-invalid"));

    let mut false_shortfall = operation("shortfall", 1, UpdateOperationStatus::Running);
    false_shortfall.missing_bytes = Some(42);
    assert_eq!(
        store.upsert(false_shortfall),
        Err("update-progress-invalid")
    );
}

#[test]
fn cancellation_cannot_be_revived_by_late_non_terminal_progress() {
    let mut store = UpdateProgressStore::default();
    store
        .upsert(operation("same", 1, UpdateOperationStatus::Running))
        .unwrap();
    let mut cancelling = operation("same", 2, UpdateOperationStatus::Cancelling);
    cancelling.can_cancel = false;
    store.upsert(cancelling).unwrap();

    assert!(!store
        .upsert(operation("same", 3, UpdateOperationStatus::Running))
        .unwrap());
    let current = store.get("same").unwrap();
    assert_eq!(current.status, UpdateOperationStatus::Cancelling);
    assert!(!current.can_cancel);
}

#[test]
fn an_error_finishing_after_cancellation_is_reported_as_cancelled() {
    let mut current = operation("same", 2, UpdateOperationStatus::Cancelling);
    current.can_cancel = false;
    let mut failed = operation("same", 3, UpdateOperationStatus::Failed);
    failed.can_cancel = false;
    failed.can_retry = true;
    failed.error_key = Some("update-download-error".into());

    super::resolve_cancelling_terminal(Some(&current), &mut failed);

    assert_eq!(failed.status, UpdateOperationStatus::Cancelled);
    assert!(!failed.can_retry);
    assert_eq!(failed.error_key, None);
}

#[test]
fn dismisses_only_a_valid_identifier() {
    let mut store = UpdateProgressStore::default();
    store
        .upsert(operation("dismiss-me", 1, UpdateOperationStatus::Failed))
        .unwrap();
    assert!(store.dismiss("dismiss-me").unwrap());
    assert!(store.snapshot().is_empty());
    assert_eq!(store.dismiss("../bad"), Err("update-progress-invalid"));

    store
        .upsert(operation("living", 1, UpdateOperationStatus::Running))
        .unwrap();
    assert_eq!(store.dismiss("living"), Err("command-not-available"));
}

#[test]
fn retry_requires_a_known_retryable_terminal_operation() {
    let runtime = super::UpdateProgressRuntime::default();
    {
        let mut store = runtime.store.lock().unwrap();
        store
            .upsert(operation("living", 1, UpdateOperationStatus::Running))
            .unwrap();
        store
            .upsert(operation("failed", 1, UpdateOperationStatus::Failed))
            .unwrap();
        let mut not_retryable = operation("cancelled", 1, UpdateOperationStatus::Cancelled);
        not_retryable.can_retry = false;
        store.upsert(not_retryable).unwrap();
    }
    assert_eq!(
        runtime.retryable("missing"),
        Err("update-progress-not-found".into())
    );
    assert!(!runtime.retryable("living").unwrap());
    assert!(runtime.retryable("failed").unwrap());
    assert!(!runtime.retryable("cancelled").unwrap());
}

#[test]
fn only_a_new_living_operation_opens_the_window() {
    let running = operation("same", 1, UpdateOperationStatus::Running);
    let failed = operation("same", 1, UpdateOperationStatus::Failed);

    assert!(super::should_show_window(None, &running));
    assert!(!super::should_show_window(Some(&running), &running));
    assert!(super::should_show_window(Some(&failed), &running));
    assert!(!super::should_show_window(None, &failed));
}

#[test]
fn checked_in_typescript_matches_the_rust_update_progress_contract() {
    let checked_in =
        include_str!("../../../../src/types/update-progress.generated.ts").replace("\r\n", "\n");
    assert_eq!(checked_in, typescript_bindings());
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_update_progress_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/update-progress.generated.ts");
    std::fs::write(path, typescript_bindings()).unwrap();
}

#[test]
fn resizing_repositions_the_window_after_applying_its_new_size() {
    let source = include_str!("window.rs");
    let resize = source
        .split_once("pub fn resize")
        .expect("resize function")
        .1;
    let size = resize.find("set_size").expect("window resize");
    let position = resize.find("set_position").expect("post-resize position");
    assert!(position > size);
}

#[test]
fn resized_window_is_clamped_inside_its_work_area() {
    use tauri::{PhysicalPosition, PhysicalSize};

    let position = super::window::clamp_to_work_area(
        PhysicalPosition::new(900, 700),
        PhysicalSize::new(464, 640),
        PhysicalPosition::new(0, 0),
        PhysicalSize::new(1_366, 768),
    );
    assert_eq!(position, PhysicalPosition::new(900, 128));
}

fn typescript_bindings() -> String {
    use ts_rs::{Config, TS};

    let config = Config::default();
    format!(
        "// @generated from Rust by `npm run contracts:generate:update-progress`.\n\
         // Do not edit this file manually.\n\n\
         export {}\n\n\
         export {}\n\n\
         export {}\n\n\
         export {}\n\n\
         export {}\n",
        UpdateOperationKind::decl(&config),
        UpdateOperationStatus::decl(&config),
        UpdateOperationPhase::decl(&config),
        UpdateProgressMode::decl(&config),
        UpdateOperationSnapshot::decl(&config),
    )
}
