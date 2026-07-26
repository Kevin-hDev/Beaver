use std::path::Path;
use std::process::Command;

const REMOVED_ENV: &[&str] = &[
    "CONDA_PREFIX",
    "HF_TOKEN",
    "HUGGINGFACE_HUB_TOKEN",
    "HUGGING_FACE_HUB_TOKEN",
    "PYTHONHOME",
    "PYTHONPATH",
    "TRANSFORMERS_CACHE",
    "VIRTUAL_ENV",
];

pub(super) fn configure(command: &mut Command, sidecar_dir: &Path) -> Result<(), String> {
    let cache = sidecar_dir.join(".hf-cache");
    match std::fs::symlink_metadata(&cache) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err("Cache Forecast invalide".to_string());
        }
        Ok(_) | Err(_) => {}
    }
    std::fs::create_dir_all(&cache)
        .map_err(|_| "Préparation du cache Forecast impossible".to_string())?;
    for key in REMOVED_ENV {
        command.env_remove(key);
    }
    command
        .env("DO_NOT_TRACK", "1")
        .env("HF_HOME", cache)
        .env("HF_HUB_DISABLE_TELEMETRY", "1")
        .env("HF_HUB_OFFLINE", "1")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("PYTHONNOUSERSITE", "1")
        .env("TOKENIZERS_PARALLELISM", "false")
        .env("TRANSFORMERS_OFFLINE", "1");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::configure;
    use std::process::Command;

    #[test]
    fn forecast_process_is_forced_offline() {
        let root = tempfile::tempdir().unwrap();
        let mut command = Command::new("python");
        configure(&mut command, root.path()).unwrap();
        let env = command.get_envs().collect::<Vec<_>>();
        for key in ["HF_HUB_OFFLINE", "TRANSFORMERS_OFFLINE", "PYTHONNOUSERSITE"] {
            assert!(env.iter().any(|(candidate, value)| {
                *candidate == key && value.is_some_and(|value| value == "1")
            }));
        }
    }
}
