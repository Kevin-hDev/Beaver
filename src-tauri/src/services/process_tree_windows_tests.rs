use super::{configure, configure_tokio, kill, terminate, ProcessKind};
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

#[test]
fn tree_termination_reaps_a_confined_parent_and_child() {
    let python = crate::services::test_runtime::python().expect("runtime Python de test");
    let temp = tempfile::tempdir().unwrap();
    let pid_file = temp.path().join("tree.pid");
    let gate_file = temp.path().join("admitted.gate");
    let mut command = Command::new(python);
    // L'arbre dort bien plus longtemps que la garde d'attente plus bas. Avec des
    // durees egales, un reap defaillant resterait indetectable : l'enfant mourrait
    // de lui-meme juste avant l'expiration de la garde, et le test verdirait par
    // hasard sur le bug meme qu'il surveille.
    command
        .args([
            "-c",
            "import os,pathlib,subprocess,sys,time; gate=pathlib.Path(sys.argv[1]);\nwhile not gate.exists(): time.sleep(.01)\nchild=subprocess.Popen([sys.executable,'-c','import time;time.sleep(120)']); pathlib.Path(sys.argv[2]).write_text(f'{os.getpid()},{child.pid}'); time.sleep(120)",
        ])
        .arg(&gate_file)
        .arg(&pid_file);
    let mut parent = crate::services::owned_process::OwnedProcess::spawn(
        &mut command,
        ProcessKind::ForecastRuntime,
    )
    .expect("start confined process tree");
    std::fs::write(&gate_file, b"admitted").unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !pid_file.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    let pids = std::fs::read_to_string(&pid_file).expect("process tree started");
    let pids = pids
        .split(',')
        .map(|pid| pid.parse::<u32>().unwrap())
        .collect::<Vec<_>>();

    let started = Instant::now();
    terminate(&mut parent, ProcessKind::ForecastRuntime);

    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(parent.try_wait().unwrap().is_some());
    // La table des processus de Windows ne se vide pas dans le meme instant que la
    // terminaison : mesure faite sur cette suite, une premiere lecture voit encore
    // l'arbre environ une fois sur huit, machine au repos. La propriete testee est
    // que l'arbre disparait, pas qu'il disparaisse en zero milliseconde — le budget
    // de terminaison est deja verifie ci-dessus sur `terminate`. La garde borne
    // l'attente pour que le test echoue au lieu de pendre.
    let garde = Instant::now() + Duration::from_secs(30);
    loop {
        let mut processes = sysinfo::System::new();
        processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        if pids
            .iter()
            .all(|pid| processes.process(sysinfo::Pid::from_u32(*pid)).is_none())
        {
            break;
        }
        assert!(
            Instant::now() < garde,
            "l'arbre confine survit a sa terminaison"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn tree_termination_never_kills_an_unconfined_process() {
    let python = crate::services::test_runtime::python().expect("runtime Python de test");
    let mut outsider = Command::new(python)
        .args(["-c", "import time;time.sleep(30)"])
        .spawn()
        .expect("start unconfined process");

    kill(outsider.id(), ProcessKind::ForecastRuntime);

    assert!(outsider.try_wait().unwrap().is_none());
    outsider.kill().unwrap();
    outsider.wait().unwrap();
}
