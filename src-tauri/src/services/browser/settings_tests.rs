use super::settings::cef_settings_policy_for_platform;
use std::path::Path;

#[test]
fn cef_settings_keep_the_sandbox_and_disable_remote_debugging() {
    let profile = Path::new("/private/tmp/cl-go/browser/profile");
    let helper = Path::new(
        "/Applications/Beaver.app/Contents/Frameworks/Beaver Helper.app/Contents/MacOS/Beaver Helper",
    );

    let settings = cef_settings_policy_for_platform(profile, helper, false);

    assert!(!settings.no_sandbox);
    assert!(settings.external_message_pump);
    assert!(!settings.multi_threaded_message_loop);
    assert_eq!(settings.remote_debugging_port, None);
    assert!(settings.command_line_args_disabled);
    assert!(settings.persist_session_cookies);
    assert_eq!(settings.profile, profile);
    assert_eq!(settings.helper, helper);
}

#[test]
fn windows_uses_cef_owned_message_loop_without_weakening_the_sandbox() {
    let profile = Path::new("C:/Users/test/AppData/Local/Beaver/browser/profile");
    let helper = Path::new("C:/Program Files/Beaver/cl-go-dash.exe");

    let settings = cef_settings_policy_for_platform(profile, helper, true);

    assert!(!settings.no_sandbox);
    assert!(!settings.external_message_pump);
    assert!(settings.multi_threaded_message_loop);
    assert_eq!(settings.remote_debugging_port, None);
}
