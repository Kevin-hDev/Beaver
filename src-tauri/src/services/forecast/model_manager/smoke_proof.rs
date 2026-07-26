use super::fs_safety;
use sha2::{Digest, Sha256};
use std::path::Path;
use subtle::ConstantTimeEq;
use tokio::io::{AsyncWriteExt, BufWriter};

const MARKER: &str = ".smoke-v3";
const MAX_SOURCE_FILES: usize = 128;
const MAX_SOURCE_BYTES: u64 = 1024 * 1024;
const SHARED_SOURCE_FILES: &[&str] = &[
    "__init__.py",
    "adapter_utils.py",
    "adapters.py",
    "config_utils.py",
    "device_utils.py",
    "limits.py",
    "quantile_utils.py",
    "validation.py",
];

pub(super) async fn write(model_dir: &Path, sidecar: &Path, family_id: &str) -> Result<(), String> {
    if !fs_safety::is_real_directory(model_dir) {
        return Err(error());
    }
    let fingerprint = source_fingerprint(sidecar, family_id)?;
    let temporary = model_dir.join(".smoke-v3.tmp");
    let destination = model_dir.join(MARKER);
    fs_safety::remove_path(&temporary)
        .await
        .map_err(|_| error())?;
    let result = async {
        let file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .await
            .map_err(|_| error())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&fingerprint).await.map_err(|_| error())?;
        writer.flush().await.map_err(|_| error())?;
        writer.get_ref().sync_all().await.map_err(|_| error())?;
        drop(writer);
        fs_safety::remove_path(&destination)
            .await
            .map_err(|_| error())?;
        tokio::fs::rename(&temporary, &destination)
            .await
            .map_err(|_| error())
    }
    .await;
    if result.is_err() {
        let _ = fs_safety::remove_path(&temporary).await;
    }
    result
}

pub(super) fn is_current(model_dir: &Path, sidecar: &Path, family_id: &str) -> bool {
    if !fs_safety::is_real_directory(model_dir) {
        return false;
    }
    let path = model_dir.join(MARKER);
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        return false;
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != 32 {
        return false;
    }
    let Ok(actual) = std::fs::read(path) else {
        return false;
    };
    let Ok(expected) = source_fingerprint(sidecar, family_id) else {
        return false;
    };
    actual.ct_eq(&expected).into()
}

fn source_fingerprint(sidecar: &Path, family_id: &str) -> Result<[u8; 32], String> {
    let runtime = sidecar.join("forecast_runtime");
    let files = source_files(family_id)?;
    let mut digest = Sha256::new();
    for name in files {
        let path = runtime.join(name);
        let metadata = std::fs::symlink_metadata(&path).map_err(|_| error())?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_SOURCE_BYTES
        {
            return Err(error());
        }
        let body = std::fs::read(&path).map_err(|_| error())?;
        digest.update((name.len() as u64).to_be_bytes());
        digest.update(name.as_bytes());
        digest.update((body.len() as u64).to_be_bytes());
        digest.update(body);
    }
    Ok(digest.finalize().into())
}

fn source_files(family_id: &str) -> Result<Vec<&'static str>, String> {
    let family_files: &[&str] = match family_id {
        "chronos-bolt" | "chronos-2" => &["chronos_adapter.py"],
        "timesfm-2-5" => &["timesfm_adapter.py", "timesfm_output.py"],
        "toto-2" => &["toto_adapter.py", "toto_multivariate.py"],
        "moirai-2" => &["moirai_adapter.py"],
        "flowstate" => &["flowstate_adapter.py"],
        "tabpfn-ts" => &["tabpfn_adapter.py"],
        "tirex" => &["tirex_adapter.py"],
        "kairos" => &["kairos_adapter.py"],
        "sundial" => &["sundial_adapter.py"],
        _ => return Err(error()),
    };
    let total = SHARED_SOURCE_FILES.len() + family_files.len();
    if total > MAX_SOURCE_FILES {
        return Err(error());
    }
    let mut files = Vec::with_capacity(total);
    files.extend_from_slice(SHARED_SOURCE_FILES);
    files.extend_from_slice(family_files);
    Ok(files)
}

fn error() -> String {
    "Validation du modèle Forecast impossible".to_string()
}

#[cfg(test)]
mod tests {
    use super::{is_current, write};

    #[tokio::test]
    async fn proof_only_changes_with_its_family_or_shared_source() {
        let root = tempfile::tempdir().unwrap();
        let runtime = root.path().join("forecast_runtime");
        let model = root.path().join("model");
        std::fs::create_dir_all(&runtime).unwrap();
        std::fs::create_dir_all(&model).unwrap();
        for file in super::source_files("chronos-bolt").unwrap() {
            std::fs::write(runtime.join(file), "first").unwrap();
        }
        std::fs::write(runtime.join("timesfm_adapter.py"), "first").unwrap();
        write(&model, root.path(), "chronos-bolt").await.unwrap();
        assert!(is_current(&model, root.path(), "chronos-bolt"));

        std::fs::write(runtime.join("timesfm_adapter.py"), "second").unwrap();
        assert!(is_current(&model, root.path(), "chronos-bolt"));

        std::fs::write(runtime.join("chronos_adapter.py"), "second").unwrap();
        assert!(!is_current(&model, root.path(), "chronos-bolt"));
    }

    #[test]
    fn every_local_catalog_family_has_a_scoped_source_list() {
        for model in crate::services::forecast::catalog::FORECAST_MODELS
            .iter()
            .filter(|model| !model.is_cloud)
        {
            assert!(
                super::source_files(model.family_id).is_ok(),
                "missing source list for {}",
                model.family_id
            );
        }
    }
}
