use std::path::PathBuf;

/// Installe une copie exécutable d'un binaire système à `destination`.
///
/// Les binaires de /usr/bin sont signés par Apple, et leur signature ne vaut
/// plus hors de leur emplacement d'origine : macOS tue la copie par SIGKILL dès
/// l'exec, alors que Linux l'exécute sans rien demander. Une signature ad hoc
/// rétablit le droit d'exécution.
#[cfg(all(test, unix))]
pub(crate) fn install_system_binary(source: &str, destination: &std::path::Path) {
    std::fs::copy(source, destination).expect("copie du binaire de test");
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("/usr/bin/codesign")
            .args(["-f", "-s", "-"])
            .arg(destination)
            .status()
            .expect("codesign disponible");
        assert!(status.success(), "signature ad hoc du binaire de test");
    }
}

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
