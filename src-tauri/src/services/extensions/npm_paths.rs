use std::path::{Path, PathBuf};

pub(super) fn resolve_cli(host_directory: &Path, _node: &Path) -> Result<PathBuf, String> {
    let bundled = host_directory.join("runtime/npm/bin/npm-cli.js");
    if bundled.is_file() {
        return bundled
            .canonicalize()
            .map(super::host_paths::node_compatible_path)
            .map_err(|_| "Gestionnaire npm indisponible.".to_string());
    }
    #[cfg(test)]
    let bin = _node
        .parent()
        .ok_or_else(|| "Gestionnaire npm indisponible.".to_string())?;
    #[cfg(test)]
    #[cfg(windows)]
    let inferred = bin.join("node_modules/npm/bin/npm-cli.js");
    #[cfg(test)]
    #[cfg(not(windows))]
    let inferred = bin.join("../lib/node_modules/npm/bin/npm-cli.js");
    #[cfg(test)]
    if let Ok(candidate) = inferred.canonicalize() {
        if candidate.is_file()
            && candidate.file_name().and_then(|name| name.to_str()) == Some("npm-cli.js")
        {
            return Ok(super::host_paths::node_compatible_path(candidate));
        }
    }
    Err("Gestionnaire npm indisponible.".to_string())
}
