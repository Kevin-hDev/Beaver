use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

const STAMP: &str = ".runtime.stamp";

pub async fn ensure_runtime(source: &Path) -> Result<PathBuf, String> {
    let source = source.to_path_buf();
    tokio::task::spawn_blocking(move || ensure_runtime_blocking(&source))
        .await
        .map_err(|_| "SearXNG: runtime indisponible".to_string())?
}

fn ensure_runtime_blocking(source: &Path) -> Result<PathBuf, String> {
    validate_source(source)?;
    let venv = super::paths::venv_dir();
    let python = venv_python(&venv);
    if !python.exists() {
        let base_python = find_python()?;
        run(
            Command::new(base_python).args(["-m", "venv"]).arg(&venv),
            "venv",
        )?;
    }

    let stamp_path = venv.join(STAMP);
    let stamp = source_stamp(source)?;
    let installed = std::fs::read_to_string(&stamp_path).unwrap_or_default();
    if installed != stamp {
        install_build_tools(&python, source)?;
        install_requirements(&python, source)?;
        std::fs::write(stamp_path, stamp)
            .map_err(|_| "SearXNG: validation runtime impossible".to_string())?;
    }
    Ok(python)
}

fn validate_source(source: &Path) -> Result<(), String> {
    let required = ["setup.py", "requirements.txt", "LICENSE", "searx/webapp.py"];
    if required.iter().all(|file| source.join(file).exists()) {
        return Ok(());
    }
    Err("SearXNG: bundle incomplet".to_string())
}

fn source_stamp(source: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();
    for file in ["setup.py", "requirements.txt", "searx/version.py"] {
        let body = std::fs::read(source.join(file))
            .map_err(|_| "SearXNG: bundle incomplet".to_string())?;
        hasher.update(body);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn install_build_tools(python: &Path, source: &Path) -> Result<(), String> {
    if let Some(wheels) = wheelhouse_for_source(source) {
        run(
            Command::new(python)
                .args(["-m", "pip", "install", "--no-index", "--find-links"])
                .arg(wheels)
                .args(["setuptools", "wheel"]),
            "outils Python",
        )
    } else {
        run(
            Command::new(python).args([
                "-m",
                "pip",
                "install",
                "--upgrade",
                "pip",
                "setuptools",
                "wheel",
            ]),
            "outils Python",
        )
    }
}

fn install_requirements(python: &Path, source: &Path) -> Result<(), String> {
    let requirements = source.join("requirements.txt");
    if let Some(wheels) = wheelhouse_for_source(source) {
        run(
            Command::new(python)
                .args(["-m", "pip", "install", "--no-index", "--find-links"])
                .arg(wheels)
                .args(["-r"])
                .arg(requirements),
            "dépendances Python",
        )
    } else {
        run(
            Command::new(python)
                .args(["-m", "pip", "install", "-r"])
                .arg(requirements),
            "dépendances Python",
        )
    }
}

fn wheelhouse_for_source(source: &Path) -> Option<PathBuf> {
    super::wheels::for_source(source)
}

fn find_python() -> Result<PathBuf, String> {
    for candidate in [
        "python3.13",
        "python3.12",
        "python3.11",
        "python3.10",
        "python3",
        "python",
    ] {
        if let Ok(path) = which::which(candidate) {
            return Ok(path);
        }
    }
    Err("SearXNG: runtime Python introuvable".to_string())
}

fn venv_python(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

fn run(command: &mut Command, step: &str) -> Result<(), String> {
    command.env("PIP_DISABLE_PIP_VERSION_CHECK", "1");
    command.env("PIP_NO_INPUT", "1");
    command.env("PYTHONUNBUFFERED", "1");
    let output = command
        .output()
        .map_err(|_| "SearXNG: runtime indisponible".to_string())?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "SearXNG: installation runtime échouée ({})",
        safe_failure_hint(&output.stderr, step)
    ))
}

fn safe_failure_hint(stderr: &[u8], step: &str) -> String {
    let text = String::from_utf8_lossy(stderr);
    for marker in [
        "No module named",
        "No matching distribution found",
        "Could not find a version",
    ] {
        if let Some(line) = text.lines().find(|line| line.contains(marker)) {
            return truncate(line.trim(), 180);
        }
    }
    step.to_string()
}

fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let mut out: String = input.chars().take(max_chars).collect();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_hint_keeps_safe_python_reason() {
        let hint = safe_failure_hint(b"ModuleNotFoundError: No module named 'msgspec'", "SearXNG");
        assert!(hint.contains("No module named"));
    }
}
