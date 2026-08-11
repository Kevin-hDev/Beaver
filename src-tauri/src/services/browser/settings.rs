use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CefSettingsPolicy {
    pub(super) profile: PathBuf,
    pub(super) browser_subprocess_path: PathBuf,
    pub(super) no_sandbox: bool,
    pub(super) external_message_pump: bool,
    pub(super) multi_threaded_message_loop: bool,
    pub(super) command_line_args_disabled: bool,
    pub(super) persist_session_cookies: bool,
    pub(super) remote_debugging_port: Option<u16>,
}

pub(super) fn cef_settings_policy(profile: &Path, helper: &Path) -> CefSettingsPolicy {
    cef_settings_policy_for_platform(profile, helper, cfg!(target_os = "windows"))
}

pub(super) fn cef_settings_policy_for_platform(
    profile: &Path,
    helper: &Path,
    windows: bool,
) -> CefSettingsPolicy {
    CefSettingsPolicy {
        profile: profile.to_path_buf(),
        // CEF force explicitement --no-sandbox sous Windows lorsqu'un chemin
        // d'exécutable enfant séparé est fourni. Le bootstrap courant est déjà
        // le bon hôte : un chemin vide laisse CEF le relancer sans désactiver
        // son sandbox. macOS conserve son bundle helper dédié.
        browser_subprocess_path: if windows {
            PathBuf::new()
        } else {
            helper.to_path_buf()
        },
        no_sandbox: false,
        external_message_pump: !windows,
        multi_threaded_message_loop: windows,
        command_line_args_disabled: true,
        persist_session_cookies: true,
        remote_debugging_port: None,
    }
}
