use serde_json::{Map, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

pub const BOOTSTRAP_FILE_MAX_BYTES: usize = 256;
pub const MAX_BOOTSTRAPPED_CONTRACT_BYTES: usize = 1_048_576;

pub fn read_bounded(path: &Path, limit: usize, overflow: &str) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|_| "cannot read contract input".to_string())?;
    let mut bytes = Vec::with_capacity(limit.min(8_192).saturating_add(1));
    file.take(limit.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read contract input".to_string())?;
    if bytes.len() > limit {
        return Err(overflow.to_string());
    }
    Ok(bytes)
}

pub fn load_contract(
    directory: &Path,
    subject: &str,
    contract_overflow: &str,
) -> Result<Value, String> {
    let bootstrap = read_bounded(
        &directory.join("contract-bootstrap.json"),
        BOOTSTRAP_FILE_MAX_BYTES,
        &format!("{subject} bootstrap exceeds its size limit"),
    )?;
    let bootstrap: Map<String, Value> = serde_json::from_slice(&bootstrap)
        .map_err(|_| format!("invalid {subject} bootstrap JSON"))?;
    if bootstrap.len() != 1 {
        return Err(format!("invalid {subject} bootstrap"));
    }
    let maximum = bootstrap
        .get("maxContractBytes")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| (1..=MAX_BOOTSTRAPPED_CONTRACT_BYTES).contains(value))
        .ok_or_else(|| format!("invalid {subject} bootstrap"))?;
    let raw = read_bounded(&directory.join("contract.json"), maximum, contract_overflow)?;
    serde_json::from_slice(&raw).map_err(|_| format!("invalid {subject} JSON"))
}

pub fn write(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|_| "cannot write generated artifact".to_string())
}

pub fn update_document(
    path: &Path,
    section: &str,
    begin_marker: &str,
    end_marker: &str,
) -> Result<(), String> {
    let source = read_bounded(path, 1_048_576, "generated document exceeds its size limit")?;
    let source = String::from_utf8(source)
        .map_err(|_| "generated document is not valid UTF-8".to_string())?;
    let updated = match (source.find(begin_marker), source.find(end_marker)) {
        (Some(begin), Some(end)) if begin < end => {
            format!(
                "{}{}{}",
                &source[..begin],
                section,
                &source[end + end_marker.len()..]
            )
        }
        (None, None) => format!("{}\n\n{}\n", source.trim_end(), section),
        _ => return Err("invalid generated document markers".to_string()),
    };
    write(path, &updated)
}

pub fn private_api_path(manifest_root: &Path) -> Result<PathBuf, String> {
    let checkout = manifest_root
        .parent()
        .ok_or_else(|| "invalid manifest root".to_string())?;
    // Generated documentation belongs to the active checkout. Following a
    // linked worktree's .git pointer would mutate another checkout silently.
    Ok(checkout.join("docs/fonctionnalites/extension/EXTENSION_API_V1.md"))
}
