use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

fn pid_file_path() -> PathBuf {
    crate::services::paths::data_dir().join("ollama-sidecar.pid")
}

pub fn save_pid(pid: u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let content = format!("{pid}:{now}");
    let tmp = pid_file_path().with_extension("tmp");
    if std::fs::write(&tmp, &content).is_ok() {
        let _ = std::fs::rename(&tmp, pid_file_path());
    }
}

pub fn read_saved_pid() -> Option<u32> {
    let content = std::fs::read_to_string(pid_file_path()).ok()?;
    let pid_str = content.trim().split(':').next()?;
    let pid: u32 = pid_str.parse().ok()?;
    if pid < 2 {
        return None;
    }
    Some(pid)
}

pub fn clear_pid_file() {
    let _ = std::fs::remove_file(pid_file_path());
}

pub fn kill_orphan_sidecar() {
    let Some(pid) = read_saved_pid() else { return };
    clear_pid_file();

    if !is_ollama_process(pid) {
        eprintln!("[ollama] pid={pid} n'est plus ollama, ignoré");
        return;
    }

    if !is_ollama_process(pid) {
        eprintln!("[ollama] pid={pid} changé entre check et kill, abandon");
        return;
    }
    eprintln!("[ollama] orphelin détecté pid={pid}, kill");
    crate::services::process_tree::kill(pid, crate::services::process_tree::ProcessKind::Ollama);
}

fn is_ollama_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().contains("ollama"),
            Err(_) => false,
        }
    }
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output();
        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .contains("ollama"),
            Err(_) => false,
        }
    }
}

pub fn kill_process(child: &mut Child) {
    let pid = child.id();
    eprintln!("[ollama] kill sidecar pid={pid}");
    crate::services::process_tree::kill(pid, crate::services::process_tree::ProcessKind::Ollama);

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(Some(_)) = child.try_wait() {
            eprintln!("[ollama] sidecar arrêté proprement");
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let _ = child.kill();
    let _ = child.wait();
}

pub fn release_vram_blocking() {
    let base_url = crate::services::ollama_port::base_url();
    let url = format!("{base_url}/api/generate");
    let body = serde_json::json!({ "model": "", "keep_alive": "0" });
    let _ = std::thread::Builder::new()
        .name("vram-release".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            if let Ok(rt) = rt {
                let _ = rt.block_on(async {
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(3))
                        .build()?;
                    client.post(&url).json(&body).send().await
                });
            }
            eprintln!("[ollama] VRAM release demandée");
        });
    std::thread::sleep(Duration::from_millis(500));
}
