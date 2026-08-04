use super::{configure, configure_tokio};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::Command;
use std::time::{Duration, Instant};
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, IsWindowVisible};

#[test]
fn configured_background_process_has_no_visible_console() {
    let title = format!("BeaverBackgroundConsoleStdTest{}", std::process::id());
    let script = format!("$host.UI.RawUI.WindowTitle='{title}'; Start-Sleep -Seconds 4");
    let mut command = Command::new("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    configure(&mut command);

    let mut child = command.spawn().expect("start fixed PowerShell fixture");
    let visible = console_is_visible(&title);

    let _ = child.kill();
    let _ = child.wait();
    assert!(!visible, "background command opened a visible console");
}

#[tokio::test]
async fn configured_tokio_background_process_has_no_visible_console() {
    let title = format!("BeaverBackgroundConsoleTokioTest{}", std::process::id());
    let script = format!("$host.UI.RawUI.WindowTitle='{title}'; Start-Sleep -Seconds 4");
    let mut command = tokio::process::Command::new("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    configure_tokio(&mut command);

    let mut child = command.spawn().expect("start Tokio PowerShell fixture");
    let visible = console_is_visible(&title);

    let _ = child.kill().await;
    assert!(!visible, "background command opened a visible console");
}

fn console_is_visible(title: &str) -> bool {
    let wide_title: Vec<u16> = OsStr::new(&title).encode_wide().chain(Some(0)).collect();
    let deadline = Instant::now() + Duration::from_secs(2);

    while Instant::now() < deadline {
        let window = unsafe { FindWindowW(std::ptr::null(), wide_title.as_ptr()) };
        if !window.is_null() && unsafe { IsWindowVisible(window) } != 0 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}
