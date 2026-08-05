use super::configure_process_group;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::{Duration, Instant};
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, IsWindowVisible};

#[tokio::test]
async fn agent_shell_process_group_has_no_visible_console() {
    let title = format!("BeaverAgentShellTest{}", std::process::id());
    let script = format!("$host.UI.RawUI.WindowTitle='{title}'; Start-Sleep -Seconds 4");
    let mut command = tokio::process::Command::new("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    configure_process_group(&mut command);

    let mut child = command.spawn().expect("start agent shell fixture");
    let visible = console_is_visible(&title);
    let _ = child.kill().await;

    assert!(!visible, "agent shell opened a visible Windows console");
}

fn console_is_visible(title: &str) -> bool {
    let wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
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
