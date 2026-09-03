use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;

use super::fingerprint_paths::{
    clean_relative, clean_root, excluded_directory, reject_symlink, relative_name,
};
use super::types::ExtensionUiMode;
use super::types::{ExtensionKind, ExtensionRecord};

pub(super) struct Verification {
    pub eligible: Vec<ExtensionRecord>,
    pub revocations: BTreeMap<String, String>,
}

pub(super) fn verify_records(records: Vec<ExtensionRecord>) -> Verification {
    let mut eligible = Vec::with_capacity(records.len());
    let mut revocations = BTreeMap::new();
    for record in records {
        if record.kind == ExtensionKind::Builtin {
            eligible.push(record);
            continue;
        }
        match is_current(&record) {
            Ok(true) => eligible.push(record),
            Ok(false) => {
                revocations.insert(
                    record.manifest.id.clone(),
                    super::error_codes::FINGERPRINT_CHANGED.to_string(),
                );
            }
            Err(_) => {
                revocations.insert(
                    record.manifest.id.clone(),
                    super::error_codes::FINGERPRINT_FAILED.to_string(),
                );
            }
        }
    }
    Verification {
        eligible,
        revocations,
    }
}

pub fn calculate(record: &ExtensionRecord) -> Result<String, String> {
    calculate_bytes(record).map(hex::encode)
}

pub fn is_current(record: &ExtensionRecord) -> Result<bool, String> {
    let encoded = record.fingerprint.as_deref().ok_or_else(error)?;
    let mut expected = [0_u8; 32];
    hex::decode_to_slice(encoded, &mut expected).map_err(|_| error())?;
    let actual = calculate_bytes(record)?;
    Ok(bool::from(actual.ct_eq(&expected)))
}

pub(super) fn same_encoded(left: Option<&str>, right: Option<&str>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => bool::from(left.as_bytes().ct_eq(right.as_bytes())),
        (None, None) => true,
        _ => false,
    }
}

fn calculate_bytes(record: &ExtensionRecord) -> Result<[u8; 32], String> {
    if record.kind != ExtensionKind::Local {
        return Err(error());
    }
    let requested_root = clean_root(&record.source)?;
    reject_symlink(requested_root)?;
    let root = dunce::canonicalize(requested_root).map_err(|_| error())?;
    if !root.is_dir() {
        return Err(error());
    }
    let main = clean_relative(record.manifest.main.as_deref().ok_or_else(error)?)?;
    let entry = root.join(main);
    let manifest = super::manifest_source::manifest_path(&root);
    let mut files = collect_files(&root, &entry, manifest.as_deref())?;
    if manifest.is_none() {
        let bytes = serde_json::to_vec(&record.manifest).map_err(|_| error())?;
        files.insert("@manifest".to_string(), FingerprintInput::Memory(bytes));
    }
    include_ui_artifact(record, &mut files)?;
    hash_files(files)
}

fn include_ui_artifact(
    record: &ExtensionRecord,
    files: &mut BTreeMap<String, FingerprintInput>,
) -> Result<(), String> {
    let advanced = record
        .manifest
        .ui
        .as_ref()
        .is_some_and(|ui| ui.mode == ExtensionUiMode::Advanced);
    if !advanced {
        return record.ui_artifact.is_none().then_some(()).ok_or_else(error);
    }
    let artifact = record.ui_artifact.as_ref().ok_or_else(error)?;
    let root =
        super::ui_artifact_store::artifact_path(&record.manifest.id, &artifact.manifest_sha256)?;
    super::ui_artifact::verify_at(&root, artifact)?;
    files.insert(
        "@ui/manifest.json".to_string(),
        FingerprintInput::Memory(super::ui_artifact::manifest_bytes(artifact)?),
    );
    for output in &artifact.outputs {
        files.insert(
            format!("@ui/{}", output.name),
            FingerprintInput::File(root.join(&output.name)),
        );
    }
    if files.len() > super::types::FINGERPRINT_MAX_FILES {
        return Err(error());
    }
    Ok(())
}

enum FingerprintInput {
    File(PathBuf),
    Memory(Vec<u8>),
}

fn collect_files(
    root: &Path,
    entry: &Path,
    manifest: Option<&Path>,
) -> Result<BTreeMap<String, FingerprintInput>, String> {
    let mut files = BTreeMap::new();
    let mut found_entry = false;
    visit_directory(root, root, 0, entry, manifest, &mut files, &mut found_entry)?;
    if !found_entry {
        return Err(error());
    }
    Ok(files)
}

fn visit_directory(
    root: &Path,
    directory: &Path,
    depth: usize,
    entry: &Path,
    manifest: Option<&Path>,
    files: &mut BTreeMap<String, FingerprintInput>,
    found_entry: &mut bool,
) -> Result<(), String> {
    if depth > super::types::FINGERPRINT_MAX_DEPTH {
        return Err(error());
    }
    for child in std::fs::read_dir(directory).map_err(|_| error())? {
        let path = child.map_err(|_| error())?.path();
        let metadata = reject_symlink(&path)?;
        if metadata.is_dir() {
            if !excluded_directory(&path) {
                visit_directory(
                    root,
                    &path,
                    depth.saturating_add(1),
                    entry,
                    manifest,
                    files,
                    found_entry,
                )?;
            }
            continue;
        }
        if !metadata.is_file() {
            return Err(error());
        }
        let selected = path == entry
            || manifest.is_some_and(|manifest| path == manifest)
            || super::manifest_source::is_source_file(&path);
        if !selected {
            continue;
        }
        *found_entry |= path == entry;
        let relative = relative_name(root, &path)?;
        files.insert(relative, FingerprintInput::File(path));
        if files.len() > super::types::FINGERPRINT_MAX_FILES {
            return Err(error());
        }
    }
    Ok(())
}

fn hash_files(files: BTreeMap<String, FingerprintInput>) -> Result<[u8; 32], String> {
    if files.len() > super::types::FINGERPRINT_MAX_FILES {
        return Err(error());
    }
    let mut total = 0_usize;
    let mut digest = Sha256::new();
    for (relative, input) in files {
        let bytes = match input {
            FingerprintInput::File(path) => read_bounded(&path)?,
            FingerprintInput::Memory(bytes) => bytes,
        };
        total = total
            .checked_add(bytes.len())
            .filter(|size| *size <= super::types::FINGERPRINT_MAX_TOTAL_BYTES)
            .ok_or_else(error)?;
        digest.update((relative.len() as u64).to_be_bytes());
        digest.update(relative.as_bytes());
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(bytes);
    }
    Ok(digest.finalize().into())
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path).map_err(|_| error())?;
    if metadata.len() > super::types::FINGERPRINT_MAX_FILE_BYTES as u64 {
        return Err(error());
    }
    let file = std::fs::File::open(path).map_err(|_| error())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(super::types::FINGERPRINT_MAX_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| error())?;
    if bytes.len() > super::types::FINGERPRINT_MAX_FILE_BYTES {
        return Err(error());
    }
    Ok(bytes)
}

fn error() -> String {
    super::error_codes::FINGERPRINT_FAILED.to_string()
}
