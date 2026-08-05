use super::{new, new_tokio};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::{Duration, Instant};
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, IsWindowVisible};

#[test]
fn standard_background_command_has_no_visible_console() {
    let title = unique_title("Std");
    let script = fixture_script(&title);
    let mut command = new("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);

    let mut child = command.spawn().expect("start standard fixture");
    let visible = console_is_visible(&title);
    let _ = child.kill();
    let _ = child.wait();

    assert!(!visible, "background command opened a visible console");
}

#[tokio::test]
async fn tokio_background_command_has_no_visible_console() {
    let title = unique_title("Tokio");
    let script = fixture_script(&title);
    let mut command = new_tokio("powershell.exe");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);

    let mut child = command.spawn().expect("start Tokio fixture");
    let visible = console_is_visible(&title);
    let _ = child.kill().await;

    assert!(!visible, "background command opened a visible console");
}

fn unique_title(kind: &str) -> String {
    format!("BeaverBackground{kind}Test{}", std::process::id())
}

fn fixture_script(title: &str) -> String {
    format!("$host.UI.RawUI.WindowTitle='{title}'; Start-Sleep -Seconds 4")
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

#[test]
fn production_console_programs_use_the_background_boundary() {
    let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    collect_direct_console_launches(&source_root, &mut violations);
    assert!(
        violations.is_empty(),
        "direct Windows console launches: {}",
        violations.join(", ")
    );
}

fn collect_direct_console_launches(directory: &std::path::Path, violations: &mut Vec<String>) {
    for entry in std::fs::read_dir(directory).expect("read Rust source directory") {
        let entry = entry.expect("read Rust source entry");
        let path = entry.path();
        if path.is_dir() {
            collect_direct_console_launches(&path, violations);
            continue;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if !name.ends_with(".rs")
            || name.contains("test")
            || matches!(name, "linux.rs" | "macos.rs" | "background_command.rs")
        {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("read Rust source");
        for program in ["git", "powershell", "tasklist", "nvidia-smi", "cmd"] {
            let needle = format!("Command::new(\"{program}\")");
            if source.contains(&needle) {
                violations.push(format!("{}:{program}", path.display()));
            }
        }
    }
}
