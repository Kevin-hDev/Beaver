use super::process_role::{
    classify_native_webview, validate_browser_process_result, NativeProcessRecord,
    NativeWebViewRole,
};

#[test]
fn only_the_browser_process_result_can_continue_initialization() {
    assert!(validate_browser_process_result(-1).is_ok());
    assert!(validate_browser_process_result(0).is_err());
    assert!(validate_browser_process_result(7).is_err());
}

fn record(pid: u32, parent_pid: u32, name: &str) -> NativeProcessRecord {
    NativeProcessRecord {
        pid,
        parent_pid,
        name: name.to_string(),
    }
}

#[test]
fn windows_webview2_is_dedicated_only_below_beaver() {
    let records = [
        record(10, 1, "beaver.exe"),
        record(11, 10, "beaver-helper.exe"),
        record(12, 11, "msedgewebview2.exe"),
        record(99, 1, "msedgewebview2.exe"),
    ];

    assert_eq!(
        classify_native_webview("windows", 10, 12, &records),
        NativeWebViewRole::Dedicated
    );
    assert_eq!(
        classify_native_webview("windows", 10, 99, &records),
        NativeWebViewRole::Other
    );
}

#[test]
fn linux_webkit_children_are_dedicated_descendants() {
    let records = [
        record(20, 1, "cl-go-dash"),
        record(21, 20, "WebKitNetworkProcess"),
        record(22, 21, "WebKitWebProcess"),
    ];

    assert_eq!(
        classify_native_webview("linux", 20, 21, &records),
        NativeWebViewRole::Dedicated
    );
    assert_eq!(
        classify_native_webview("linux", 20, 22, &records),
        NativeWebViewRole::Dedicated
    );
}

#[test]
fn macos_webkit_services_are_shared_and_never_dedicated() {
    let records = [
        record(30, 1, "cl-go-dash"),
        record(31, 30, "com.apple.WebKit.WebContent"),
        record(32, 1, "com.apple.WebKit.Networking"),
    ];

    assert_eq!(
        classify_native_webview("macos", 30, 31, &records),
        NativeWebViewRole::SharedSystem
    );
    assert_eq!(
        classify_native_webview("macos", 30, 32, &records),
        NativeWebViewRole::SharedSystem
    );
}

#[test]
fn ancestry_cycles_fail_closed_as_unrelated() {
    let records = [
        record(40, 1, "beaver.exe"),
        record(41, 42, "helper.exe"),
        record(42, 41, "msedgewebview2.exe"),
    ];

    assert_eq!(
        classify_native_webview("windows", 40, 42, &records),
        NativeWebViewRole::Other
    );
}
