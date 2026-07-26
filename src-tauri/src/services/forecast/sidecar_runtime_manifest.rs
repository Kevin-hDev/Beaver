use std::path::Path;

const MAX_RUNTIME_LOCK_SIZE: usize = 512 * 1024;
const APPROVED_KAIROS_ARCHIVE: &str =
    "https://github.com/foundation-model-research/Kairos/archive/0322393840ccf6e2bfe9c663f9dcd088a5a7ee07.zip";

pub(super) fn expected_requirements(sidecar_dir: &Path, family_id: &str) -> Result<String, String> {
    let mut lock = base_requirements(sidecar_dir, family_id)?;
    if let Some(source) = source_requirements(family_id)? {
        lock.push_str("\n# audited source\n");
        lock.push_str(&source);
    }
    Ok(lock)
}

pub(super) fn base_requirements(_sidecar_dir: &Path, family_id: &str) -> Result<String, String> {
    let lock = super::lock_data::lock_for_runtime(runtime_id(family_id)?);
    validate_lock(lock)?;
    Ok(lock.to_string())
}

pub(super) fn source_requirements(family_id: &str) -> Result<Option<String>, String> {
    let Some(source) = super::lock_data::source_for_runtime(runtime_id(family_id)?) else {
        return Ok(None);
    };
    validate_lock(source)?;
    Ok(Some(source.to_string()))
}

pub(crate) fn runtime_id(family_id: &str) -> Result<&'static str, String> {
    validate_family_id(family_id)?;
    match family_id {
        "chronos-bolt" | "chronos-2" => Ok("chronos"),
        "timesfm-2-5" => Ok("timesfm"),
        "toto-2" => Ok("toto"),
        "moirai-2" => Ok("moirai"),
        "flowstate" => Ok("flowstate"),
        "tabpfn-ts" => Ok("tabpfn"),
        "tirex" => Ok("tirex"),
        "kairos" => Ok("kairos"),
        "sundial" => Ok("sundial"),
        _ => Err("Adapter Forecast indisponible".to_string()),
    }
}

pub(crate) fn locked_package_version(family_id: &str, package: &str) -> Result<String, String> {
    if package.is_empty()
        || package.len() > 80
        || !package
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err("Paquet Forecast invalide".to_string());
    }
    let lock = super::lock_data::lock_for_runtime(runtime_id(family_id)?);
    validate_lock(lock)?;
    let prefix = format!("{package}==");
    logical_requirements(lock)
        .into_iter()
        .find_map(|requirement| {
            requirement
                .strip_prefix(&prefix)
                .and_then(|tail| tail.split_whitespace().next())
                .map(str::to_string)
        })
        .ok_or_else(|| "Version Forecast introuvable".to_string())
}

pub(super) fn validate_family_id(family_id: &str) -> Result<(), String> {
    if family_id.is_empty()
        || family_id.len() > 80
        || !family_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err("Famille Forecast invalide".to_string());
    }
    Ok(())
}

fn validate_lock(lock: &str) -> Result<(), String> {
    if lock.is_empty() || lock.len() > MAX_RUNTIME_LOCK_SIZE {
        return Err("Configuration runtime Forecast invalide".to_string());
    }
    let forbidden = [
        "git+",
        "file:",
        "--index-url",
        "--extra-index-url",
        "--trusted-host",
        "-e ",
    ];
    if forbidden.iter().any(|value| lock.contains(value)) {
        return Err("Configuration runtime Forecast invalide".to_string());
    }
    for block in logical_requirements(lock) {
        let source_is_pinned = block.contains("==")
            || (block.contains(APPROVED_KAIROS_ARCHIVE) && block.starts_with("kairos @ "));
        if !source_is_pinned || !has_valid_sha256(&block) {
            return Err("Configuration runtime Forecast invalide".to_string());
        }
    }
    Ok(())
}

fn logical_requirements(lock: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for raw in lock.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(line.trim_end_matches('\\').trim_end());
        if !line.ends_with('\\') {
            blocks.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

fn has_valid_sha256(block: &str) -> bool {
    block.split("--hash=sha256:").skip(1).any(|tail| {
        let hash = tail.split_whitespace().next().unwrap_or_default();
        hash.len() == 64 && hash.chars().all(|character| character.is_ascii_hexdigit())
    })
}

#[cfg(test)]
mod tests {
    use super::{locked_package_version, logical_requirements, runtime_id, validate_lock};

    #[test]
    fn chronos_families_share_one_runtime() {
        assert_eq!(runtime_id("chronos-bolt").unwrap(), "chronos");
        assert_eq!(runtime_id("chronos-2").unwrap(), "chronos");
    }

    #[test]
    fn lock_rejects_unhashed_or_mutable_sources() {
        assert!(validate_lock("package==1.0").is_err());
        assert!(validate_lock("package @ git+https://example.invalid/repo").is_err());
        assert!(validate_lock("package==1.0 --hash=sha256:abcd").is_err());
    }

    #[test]
    fn logical_lines_keep_hashes_with_their_package() {
        let hash = "a".repeat(64);
        let body = format!("package==1.0 \\\n+  --hash=sha256:{hash}\n");
        let blocks = logical_requirements(&body);
        assert_eq!(blocks.len(), 1);
        assert!(validate_lock(&body).is_ok());
    }

    #[test]
    fn direct_version_comes_from_the_platform_lock() {
        assert_eq!(
            locked_package_version("chronos-bolt", "chronos-forecasting").unwrap(),
            "2.3.1"
        );
        assert!(locked_package_version("chronos-bolt", "../package").is_err());
    }
}
