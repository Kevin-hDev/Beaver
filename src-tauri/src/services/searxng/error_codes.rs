pub(crate) const APP_UNAVAILABLE: &str = "searxng_app_unavailable";
pub(crate) const BUNDLE_INVALID: &str = "searxng_bundle_invalid";
pub(crate) const CONFIG_UNAVAILABLE: &str = "searxng_config_unavailable";
pub(crate) const LOG_UNAVAILABLE: &str = "searxng_log_unavailable";
pub(crate) const OPERATION_INTERRUPTED: &str = "searxng_operation_interrupted";
pub(crate) const PROCESS_STATE_UNAVAILABLE: &str = "searxng_process_state_unavailable";
pub(crate) const RUNTIME_UNAVAILABLE: &str = "searxng_runtime_unavailable";
pub(crate) const SEARCH_FAILED: &str = "searxng_search_failed";
pub(crate) const SEARCH_INVALID_RESPONSE: &str = "searxng_search_invalid_response";
pub(crate) const SEARCH_RATE_LIMITED: &str = "searxng_search_rate_limited";
pub(crate) const SETTINGS_UNAVAILABLE: &str = "searxng_settings_unavailable";
pub(crate) const SHUTTING_DOWN: &str = "searxng_shutting_down";
pub(crate) const SOURCE_UNAVAILABLE: &str = "searxng_source_unavailable";
pub(crate) const START_FAILED: &str = "searxng_start_failed";

pub(crate) const ALL: [&str; 14] = [
    APP_UNAVAILABLE,
    BUNDLE_INVALID,
    CONFIG_UNAVAILABLE,
    LOG_UNAVAILABLE,
    OPERATION_INTERRUPTED,
    PROCESS_STATE_UNAVAILABLE,
    RUNTIME_UNAVAILABLE,
    SEARCH_FAILED,
    SEARCH_INVALID_RESPONSE,
    SEARCH_RATE_LIMITED,
    SETTINGS_UNAVAILABLE,
    SHUTTING_DOWN,
    SOURCE_UNAVAILABLE,
    START_FAILED,
];

pub(crate) fn known(value: &str) -> Option<&'static str> {
    ALL.into_iter().find(|code| *code == value)
}
