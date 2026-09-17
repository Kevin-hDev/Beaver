use serde_json::Value;
use std::path::Path;

pub fn has_runtime_dependencies(root: &Path) -> Result<bool, super::OperationFailure> {
    let path = root.join("package.json");
    if !path.is_file() {
        return Ok(false);
    }
    let metadata =
        std::fs::metadata(&path).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    if metadata.len() > super::types::MAX_MESSAGE_BYTES as u64 {
        return Err(super::OperationFailure::ManifestInvalid);
    }
    let bytes = std::fs::read(path).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|_| super::OperationFailure::ManifestInvalid)?;
    Ok(["dependencies", "optionalDependencies", "peerDependencies"]
        .into_iter()
        .any(|field| {
            value
                .get(field)
                .and_then(Value::as_object)
                .is_some_and(|dependencies| !dependencies.is_empty())
        }))
}
