use super::loading_marker::{self, JournalRead};
use super::ui_startup::{self, ShiftSource};
use super::ui_startup_ack::{UiAckToken, UiLoadAcknowledger};
use super::ui_startup_state::{SafeReason, UiStartupMode};

const UI_ID: &str = "com.example.ui";

#[test]
fn safe_mode_argument_is_exact_unique_and_bounded() {
    assert_eq!(ui_startup::safe_mode_from_args(["beaver"]), Ok(false));
    assert_eq!(
        ui_startup::safe_mode_from_args(["beaver", "--safe-mode"]),
        Ok(true)
    );
    assert!(ui_startup::safe_mode_from_args(["beaver", "--safe-mode", "--safe-mode"]).is_err());
    assert!(ui_startup::safe_mode_from_args(["beaver", "--safe-mode=true"]).is_err());
    assert!(ui_startup::safe_mode_from_args(["beaver", &"x".repeat(2_049)]).is_err());
}

#[test]
fn startup_argument_collection_is_bounded_before_allocation_can_grow() {
    let accepted = (0..ui_startup::MAX_STARTUP_ARGS)
        .map(|index| std::ffi::OsString::from(format!("arg-{index}")));
    assert_eq!(
        ui_startup::collect_startup_args(accepted).unwrap().len(),
        ui_startup::MAX_STARTUP_ARGS
    );
    let rejected = (0..=ui_startup::MAX_STARTUP_ARGS)
        .map(|index| std::ffi::OsString::from(format!("arg-{index}")));
    assert!(ui_startup::collect_startup_args(rejected).is_err());
}

#[test]
fn ipc_acknowledgement_token_has_one_exact_deserialized_size() {
    assert!(serde_json::from_value::<UiAckToken>(serde_json::json!(vec![0_u8; 31])).is_err());
    assert!(serde_json::from_value::<UiAckToken>(serde_json::json!(vec![0_u8; 32])).is_ok());
    assert!(serde_json::from_value::<UiAckToken>(serde_json::json!(vec![0_u8; 33])).is_err());
}

#[test]
fn cef_child_removes_only_the_exact_valueless_parent_safe_switch() {
    assert_eq!(ui_startup::cef_safe_mode_switch_name(), "safe-mode");
    assert_eq!(ui_startup::cef_child_safe_mode_action(false, ""), Ok(false));
    assert_eq!(ui_startup::cef_child_safe_mode_action(true, ""), Ok(true));
    assert!(ui_startup::cef_child_safe_mode_action(true, "true").is_err());
}

#[test]
fn cef_launch_authority_consumes_the_central_safe_mode_decision() {
    let source = include_str!("../browser/cef_child_admission.rs");
    assert!(source.contains("cef_child_safe_mode_action"));
    assert!(source.contains("command_line.remove_switch"));
    assert!(!source.contains("\"safe-mode\""));
}

#[test]
fn malformed_safe_mode_never_masquerades_as_an_invalid_journal() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    assert!(
        ui_startup::prepare_from_args_at(&path, ["beaver", "--safe-mode=true"], false, false,)
            .is_err()
    );
    assert!(matches!(
        loading_marker::read_journal_at(&path),
        JournalRead::Missing
    ));
}

#[test]
fn startup_decision_is_fixed_before_ui_loading() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");

    let normal = ui_startup::decide_at(&path, false, false).unwrap();
    assert_eq!(normal.mode(), UiStartupMode::Normal);
    assert!(normal.third_party_loading_allowed());

    loading_marker::ui_start_at(&path, UI_ID, 2).unwrap();
    let pending = ui_startup::decide_at(&path, false, false).unwrap();
    assert!(matches!(
        pending.mode(),
        UiStartupMode::PendingInterruptedUi { extension_id, stage, started_at, attempts }
            if extension_id == UI_ID && stage == "contract"
                && chrono::DateTime::parse_from_rfc3339(&started_at).is_ok()
                && attempts == 2
    ));
    assert!(!pending.third_party_loading_allowed());

    let argument = ui_startup::decide_at(&path, true, false).unwrap();
    assert_eq!(
        argument.mode(),
        UiStartupMode::Safe {
            reason: SafeReason::Argument
        }
    );
    let shift = ui_startup::decide_at(&path, false, true).unwrap();
    assert_eq!(
        shift.mode(),
        UiStartupMode::Safe {
            reason: SafeReason::Shift
        }
    );

    std::fs::write(&path, b"invalid-security-control").unwrap();
    let invalid = ui_startup::decide_at(&path, false, false).unwrap();
    assert_eq!(
        invalid.mode(),
        UiStartupMode::Safe {
            reason: SafeReason::InvalidMarker
        }
    );
}

#[test]
fn platform_shift_paths_are_testable_without_the_current_os() {
    for source in [ShiftSource::MacOs, ShiftSource::Windows, ShiftSource::X11] {
        assert!(ui_startup::probe_shift(source, || Some(true)));
        assert!(!ui_startup::probe_shift(source, || Some(false)));
        assert!(!ui_startup::probe_shift(source, || None));
    }
    assert!(!ui_startup::probe_shift(ShiftSource::Wayland, || Some(
        true
    )));
}

#[test]
fn windows_shift_native_api_feature_is_explicit() {
    let manifest = include_str!("../../../Cargo.toml");
    assert!(manifest.contains("\"Win32_UI_Input_KeyboardAndMouse\""));
    assert!(include_str!("ui_startup_platform.rs").contains("GetAsyncKeyState(VK_SHIFT"));
}

#[test]
fn wayland_confirmation_resolves_once_and_blocks_until_then() {
    let state = ui_startup::unresolved_wayland_state();
    assert!(!state.bootstrap_resolved());
    assert!(!state.third_party_loading_allowed());
    assert!(state.confirm_wayland_shift(true).is_ok());
    assert_eq!(
        state.mode(),
        UiStartupMode::Safe {
            reason: SafeReason::Shift
        }
    );
    assert!(state.confirm_wayland_shift(false).is_err());
}

#[test]
fn wayland_without_shift_resolves_the_interrupted_ui_journal() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::ui_start_at(&path, UI_ID, 2).unwrap();
    let state = ui_startup::prepare_from_args_at(&path, ["beaver"], false, true).unwrap();

    assert_eq!(state.mode(), UiStartupMode::AwaitingWayland);
    assert!(!state.third_party_loading_allowed());
    state.confirm_wayland_shift(false).unwrap();
    assert!(matches!(
        state.mode(),
        UiStartupMode::PendingInterruptedUi { extension_id, attempts, .. }
            if extension_id == UI_ID && attempts == 2
    ));
    assert!(!state.third_party_loading_allowed());
}

#[test]
fn wayland_without_shift_keeps_an_invalid_journal_fail_closed() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    std::fs::write(&path, b"invalid-security-control").unwrap();
    let state = ui_startup::prepare_from_args_at(&path, ["beaver"], false, true).unwrap();

    assert_eq!(state.mode(), UiStartupMode::AwaitingWayland);
    state.confirm_wayland_shift(false).unwrap();
    assert_eq!(
        state.mode(),
        UiStartupMode::Safe {
            reason: SafeReason::InvalidMarker,
        }
    );
    assert!(!state.third_party_loading_allowed());
}

#[test]
fn retry_authorizes_only_the_interrupted_identity_at_the_next_attempt() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::ui_start_at(&path, UI_ID, 2).unwrap();
    let state = ui_startup::decide_at(&path, false, false).unwrap();

    state.retry_pending().unwrap();
    assert!(matches!(
        state.mode(),
        UiStartupMode::RetryInterruptedUi { extension_id, attempts }
            if extension_id == UI_ID && attempts == 3
    ));
    assert!(state.loading_allowed_for(UI_ID, 3));
    assert!(!state.loading_allowed_for(UI_ID, 2));
    assert!(!state.loading_allowed_for("com.example.other", 3));
    assert!(!state.third_party_loading_allowed());

    loading_marker::ui_start_at(&path, UI_ID, 3).unwrap();
    let exhausted = ui_startup::decide_at(&path, false, false).unwrap();
    assert!(exhausted.retry_pending().is_err());
}

#[test]
fn closing_invalid_marker_dialog_keeps_marker_and_stays_safe() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    std::fs::write(&path, b"invalid-security-control").unwrap();
    let state = ui_startup::decide_at(&path, false, false).unwrap();

    state.choose_safe().unwrap();

    assert_eq!(
        state.mode(),
        UiStartupMode::Safe {
            reason: SafeReason::RecoveryChoice,
        }
    );
    assert!(!state.third_party_loading_allowed());
    assert_eq!(std::fs::read(&path).unwrap(), b"invalid-security-control");
}

#[test]
fn acknowledgement_is_random_single_use_constant_time_and_leaves_failures() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let acknowledger = UiLoadAcknowledger::for_path(path.clone());

    let first = acknowledger.begin(UI_ID, 1).unwrap();
    assert_eq!(first.len(), 32);
    assert!(acknowledger.acknowledge(UI_ID, &[0; 32]).is_err());
    assert!(matches!(
        loading_marker::read_journal_at(&path),
        JournalRead::Valid(_)
    ));
    assert!(acknowledger
        .acknowledge("com.example.other", &first)
        .is_err());
    acknowledger.acknowledge(UI_ID, &first).unwrap();
    assert!(matches!(
        loading_marker::read_journal_at(&path),
        JournalRead::Missing
    ));
    assert!(acknowledger.acknowledge(UI_ID, &first).is_err());

    let second = acknowledger.begin(UI_ID, 2).unwrap();
    assert_ne!(first, second);
    assert!(acknowledger.begin(UI_ID, 2).is_err());
    acknowledger.fail_active_attempt();
    assert!(matches!(
        loading_marker::read_journal_at(&path),
        JournalRead::Valid(_)
    ));
    assert!(acknowledger.begin(UI_ID, 4).is_err());
}

#[test]
fn rng_failure_leaves_the_security_journal_in_place() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    let acknowledger = UiLoadAcknowledger::for_path(path.clone());

    assert!(acknowledger.begin_rng_failure_for_test(UI_ID, 1).is_err());
    let JournalRead::Valid(journal) = loading_marker::read_journal_at(&path) else {
        panic!("failed attempt must remain recoverable");
    };
    assert_eq!(journal.ui().unwrap().extension_id, UI_ID);
}
