use std::path::PathBuf;

pub(crate) fn python() -> Result<PathBuf, String> {
    let candidates = if cfg!(windows) {
        ["python", "python3"]
    } else {
        ["python3", "python"]
    };
    candidates
        .into_iter()
        .filter_map(|candidate| which::which(candidate).ok())
        .find(|path| {
            std::process::Command::new(path)
                .arg("--version")
                .output()
                .is_ok_and(|output| output.status.success())
        })
        .ok_or_else(|| "runtime Python de test indisponible".to_string())
}
