use super::*;
use std::sync::Mutex;

/// Sérialise les tests qui modifient les états globaux SearXNG.
static GLOBAL_STATE_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn safe_log_error_removes_control_chars_and_truncates() {
    let input = format!("SearXNG: timeout\n{}", "x".repeat(400));
    let output = safe_log_error(&input);
    assert!(!output.contains('\n'));
    assert!(output.chars().count() <= 240);
}

#[test]
fn start_failure_cache_expires() {
    let _guard = GLOBAL_STATE_LOCK.lock().unwrap();
    clear_start_failure();
    remember_start_failure("SearXNG: arrêt au démarrage");
    assert_eq!(
        recent_start_failure(),
        Some("SearXNG: arrêt au démarrage".to_string())
    );
    clear_start_failure();
    assert_eq!(recent_start_failure(), None);
}

#[test]
fn is_ready_reads_flag_and_failure_clears_it() {
    let _guard = GLOBAL_STATE_LOCK.lock().unwrap();
    clear_start_failure();
    set_ready(false);

    set_ready(true);
    assert!(is_ready());

    remember_start_failure("SearXNG: timeout au démarrage");
    assert!(!is_ready());

    clear_start_failure();
    set_ready(false);
}
