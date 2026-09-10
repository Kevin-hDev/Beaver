pub(crate) fn models_directory_path() -> Option<std::path::PathBuf> {
    let paths = crate::services::paths::ollama_paths(&crate::services::paths::data_dir());
    super::recovery_entry::frozen_models_directory(&paths)
        .map(|directory| directory.path().to_path_buf())
}

pub(crate) fn process_name_matches(name: &str) -> bool {
    if name == "ollama" || name.eq_ignore_ascii_case("ollama.exe") {
        return true;
    }
    #[cfg(unix)]
    return super::spawn_gate_unix::is_process_name(name);
    #[cfg(not(unix))]
    false
}
