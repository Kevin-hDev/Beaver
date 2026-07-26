use super::settings::CefSettingsPolicy;
use cef::{CefString, LogSeverity, Settings};

pub(super) fn prepare_profile() -> Result<std::path::PathBuf, ()> {
    let browser_root = crate::services::paths::data_dir().join("browser");
    let profile = browser_root.join("profile");
    std::fs::create_dir_all(&profile).map_err(|_| ())?;
    crate::services::private_store::repair_path(&browser_root).map_err(|_| ())?;
    crate::services::private_store::repair_path(&profile).map_err(|_| ())?;
    Ok(profile)
}

pub(super) fn to_cef_settings(policy: CefSettingsPolicy) -> Settings {
    Settings {
        no_sandbox: i32::from(policy.no_sandbox),
        browser_subprocess_path: CefString::from(policy.helper.to_string_lossy().as_ref()),
        multi_threaded_message_loop: i32::from(policy.multi_threaded_message_loop),
        external_message_pump: i32::from(policy.external_message_pump),
        command_line_args_disabled: i32::from(policy.command_line_args_disabled),
        cache_path: CefString::from(policy.profile.to_string_lossy().as_ref()),
        root_cache_path: CefString::from(policy.profile.to_string_lossy().as_ref()),
        persist_session_cookies: i32::from(policy.persist_session_cookies),
        log_severity: LogSeverity::DISABLE,
        remote_debugging_port: policy.remote_debugging_port.map(i32::from).unwrap_or(0),
        ..Default::default()
    }
}
