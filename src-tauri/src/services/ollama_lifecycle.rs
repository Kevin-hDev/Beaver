use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::services::gpu_detect;
use crate::services::ollama_kill;
use crate::services::ollama_port;

pub struct OllamaSidecar(pub Mutex<Option<Child>>);

impl OllamaSidecar {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

pub fn ollama_bundle_dir() -> PathBuf {
    crate::services::paths::data_dir().join("ollama-bundle")
}

pub fn ollama_binary_path() -> Result<PathBuf, String> {
    let binary_name = if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    };
    let bundle = ollama_bundle_dir();
    let in_bin = bundle.join("bin").join(binary_name);
    if in_bin.exists() {
        return Ok(in_bin);
    }
    let at_root = bundle.join(binary_name);
    if at_root.exists() {
        return Ok(at_root);
    }
    Err("ollama-not-installed".into())
}

pub fn is_ollama_ready() -> bool {
    ollama_port::detect_existing_instance(ollama_port::get_port())
        || ollama_port::detect_existing_instance(11434)
}

pub fn is_ollama_installed_or_external() -> bool {
    ollama_binary_path().is_ok() || is_ollama_ready()
}

pub fn start_sidecar(app: &AppHandle) -> Result<bool, String> {
    let state = app.state::<OllamaSidecar>();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(child) = guard.as_mut() {
        match child.try_wait() {
            Ok(None) => return Ok(false),
            Ok(Some(_)) => {
                *guard = None;
            }
            Err(e) => {
                eprintln!("[ollama] sidecar status: {e}");
                return Err("ollama-status-error".to_string());
            }
        }
    }

    ollama_kill::kill_orphan_sidecar();

    let port = ollama_port::find_free_port();

    if ollama_port::detect_existing_instance(port) {
        eprintln!("[ollama] daemon existant détecté sur {port}, sidecar ignoré");
        ollama_port::set_port(port);
        return Ok(false);
    }

    if ollama_port::detect_existing_instance(11434) {
        eprintln!("[ollama] daemon système détecté sur 11434, réutilisation");
        ollama_port::set_port(11434);
        return Ok(false);
    }

    ollama_port::set_port(port);
    eprintln!("[ollama] port sélectionné : {port}");

    let binary = ollama_binary_path()?;
    let bundle_dir = ollama_bundle_dir();

    let log_dir = crate::services::paths::data_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let stderr_file = std::fs::File::create(log_dir.join("ollama-sidecar.log")).map_err(|e| {
        eprintln!("[ollama] log file: {e}");
        "ollama-log-error".to_string()
    })?;

    let mut cmd = Command::new(&binary);
    cmd.arg("serve")
        .current_dir(&bundle_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));

    let config = crate::services::config::read_config().unwrap_or_default();
    let gpu = gpu_detect::detect();
    let env_vars = crate::services::ollama_env::build_env_vars(&config.advanced, &gpu, port);
    for (key, val) in &env_vars {
        cmd.env(key, val);
    }
    eprintln!(
        "[ollama] GPU : {:?}, accel : {}",
        gpu, config.advanced.hardware_accel
    );
    const SAFE_TO_LOG: &[&str] = &[
        "OLLAMA_HOST",
        "OLLAMA_FLASH_ATTENTION",
        "OLLAMA_KV_CACHE_TYPE",
        "OLLAMA_NUM_PARALLEL",
        "OLLAMA_NO_CLOUD",
        "OLLAMA_LOAD_TIMEOUT",
        "OLLAMA_CONTEXT_LENGTH",
        "OLLAMA_MAX_LOADED_MODELS",
        "OLLAMA_KEEP_ALIVE",
        "OLLAMA_LLM_LIBRARY",
        "OLLAMA_GPU_OVERHEAD",
        "OLLAMA_VULKAN",
    ];
    let safe_vars: Vec<_> = env_vars
        .iter()
        .filter(|(k, _)| SAFE_TO_LOG.contains(&k.as_str()))
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    eprintln!("[ollama] env : {:?}", safe_vars);

    #[cfg(target_os = "linux")]
    {
        let lib_dir = bundle_dir.join("lib/ollama");
        if lib_dir.is_dir() {
            let existing = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
            let new_path = format!("{}:{existing}", lib_dir.display());
            cmd.env("LD_LIBRARY_PATH", new_path);
            eprintln!("[ollama] LD_LIBRARY_PATH prépend {}", lib_dir.display());
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let child = cmd.spawn().map_err(|e| {
        eprintln!("[ollama] spawn: {e}");
        "ollama-start-error".to_string()
    })?;
    let pid = child.id();
    ollama_kill::save_pid(pid);
    eprintln!("[ollama] sidecar démarré pid={pid} port={port}");

    *guard = Some(child);

    Ok(true)
}

pub fn stop_sidecar(app: &AppHandle) {
    let state = app.state::<OllamaSidecar>();
    let mut guard = match state.0.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some(mut child) = guard.take() {
        ollama_kill::kill_process(&mut child);
    }
    ollama_kill::clear_pid_file();
}
