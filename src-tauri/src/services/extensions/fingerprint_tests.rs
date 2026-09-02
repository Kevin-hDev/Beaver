use super::fingerprint;
use super::types::{ExtensionKind, ExtensionRecord};

fn record(root: &std::path::Path) -> ExtensionRecord {
    let mut record = super::builtin::records().unwrap().remove(0);
    record.kind = ExtensionKind::Local;
    record.manifest.id = "com.example.fingerprint".to_string();
    record.manifest.main = Some("index.mjs".to_string());
    record.source = root.to_str().unwrap().to_string();
    record.origin = None;
    record
}

fn source_tree() -> (tempfile::TempDir, ExtensionRecord) {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("index.mjs"), "export default {};\n").unwrap();
    std::fs::write(
        directory.path().join("helper.ts"),
        "export const value = 1;\n",
    )
    .unwrap();
    let record = record(directory.path());
    (directory, record)
}

#[test]
fn source_addition_removal_and_modification_change_the_fingerprint() {
    let (directory, record) = source_tree();
    let initial = fingerprint::calculate(&record).unwrap();

    std::fs::write(directory.path().join("added.js"), "export {};\n").unwrap();
    let added = fingerprint::calculate(&record).unwrap();
    assert_ne!(added, initial);
    std::fs::remove_file(directory.path().join("added.js")).unwrap();
    assert_eq!(fingerprint::calculate(&record).unwrap(), initial);
    std::fs::write(
        directory.path().join("helper.ts"),
        "export const value = 2;\n",
    )
    .unwrap();
    assert_ne!(fingerprint::calculate(&record).unwrap(), initial);
}

#[test]
fn dependency_and_git_metadata_changes_do_not_change_the_fingerprint() {
    let (directory, record) = source_tree();
    let initial = fingerprint::calculate(&record).unwrap();
    std::fs::create_dir(directory.path().join("node_modules")).unwrap();
    std::fs::create_dir(directory.path().join(".git")).unwrap();
    std::fs::write(directory.path().join("node_modules/dependency.js"), "first").unwrap();
    std::fs::write(directory.path().join(".git/index"), "first").unwrap();

    assert_eq!(fingerprint::calculate(&record).unwrap(), initial);
    std::fs::write(
        directory.path().join("node_modules/dependency.js"),
        "second",
    )
    .unwrap();
    std::fs::write(directory.path().join(".git/index"), "second").unwrap();
    assert_eq!(fingerprint::calculate(&record).unwrap(), initial);
}

#[test]
fn a_directory_named_like_a_manifest_cannot_replace_the_manifest_input() {
    let (directory, record) = source_tree();
    let initial = fingerprint::calculate(&record).unwrap();
    std::fs::create_dir(directory.path().join("beaver-extension.json")).unwrap();

    assert_eq!(fingerprint::calculate(&record).unwrap(), initial);
}

#[test]
fn comparison_uses_the_stored_sha256_and_fails_closed_on_invalid_values() {
    let (_directory, mut record) = source_tree();
    record.fingerprint = Some(fingerprint::calculate(&record).unwrap());
    assert!(fingerprint::is_current(&record).unwrap());
    record.fingerprint = Some("00".repeat(32));
    assert!(!fingerprint::is_current(&record).unwrap());
    record.fingerprint = Some("invalid".to_string());
    assert!(fingerprint::is_current(&record).is_err());
}

#[test]
fn source_root_traversal_is_rejected_before_canonicalization() {
    let (directory, mut record) = source_tree();
    std::fs::create_dir(directory.path().join("nested")).unwrap();
    record.source = directory
        .path()
        .join("nested/..")
        .to_str()
        .unwrap()
        .to_string();

    assert!(fingerprint::calculate(&record).is_err());
}

#[test]
fn fingerprint_relative_names_use_one_cross_platform_separator() {
    let directory = tempfile::tempdir().unwrap();
    let nested = directory.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    let file = nested.join("file.ts");
    std::fs::write(&file, "").unwrap();

    assert_eq!(
        super::fingerprint_paths::relative_name(directory.path(), &file).unwrap(),
        "nested/file.ts"
    );
}

#[test]
fn approval_records_the_current_fingerprint_and_timestamp() {
    let (_directory, mut record) = source_tree();
    record.trusted = false;
    record.fingerprint = None;
    let timestamp = "2026-09-02T10:00:00Z";

    super::registry_state::approve_local(&mut record, timestamp).unwrap();

    assert!(record.trusted);
    assert_eq!(record.trusted_at.as_deref(), Some(timestamp));
    assert!(fingerprint::is_current(&record).unwrap());
}

#[test]
fn changed_and_unreadable_extensions_are_revoked_before_specs_are_built() {
    let (changed_directory, mut changed) = source_tree();
    changed.manifest.id = "com.example.changed".to_string();
    changed.enabled = true;
    changed.trusted = true;
    changed.fingerprint = Some(fingerprint::calculate(&changed).unwrap());
    std::fs::write(changed_directory.path().join("helper.ts"), "changed\n").unwrap();

    let (_healthy_directory, mut healthy) = source_tree();
    healthy.manifest.id = "com.example.healthy".to_string();
    healthy.enabled = true;
    healthy.trusted = true;
    healthy.fingerprint = Some(fingerprint::calculate(&healthy).unwrap());

    let (_missing_directory, mut missing) = source_tree();
    missing.manifest.id = "com.example.missing".to_string();
    missing.enabled = true;
    missing.trusted = true;
    missing.source = "/definitely/missing/beaver-extension".to_string();
    missing.fingerprint = Some("00".repeat(32));

    let verified = fingerprint::verify_records(vec![changed, healthy, missing]);

    assert_eq!(verified.eligible.len(), 1);
    assert_eq!(verified.eligible[0].manifest.id, "com.example.healthy");
    assert_eq!(
        verified.revocations["com.example.changed"],
        super::error_codes::FINGERPRINT_CHANGED
    );
    assert_eq!(
        verified.revocations["com.example.missing"],
        super::error_codes::FINGERPRINT_FAILED
    );
}

#[test]
fn revocation_closes_the_record_and_requests_the_sensitive_access_reminder() {
    let (_directory, mut record) = source_tree();
    record.enabled = true;
    record.trusted = true;
    record.trusted_at = Some("2026-09-01T10:00:00Z".to_string());
    record.sensitive_access_granted = true;
    let id = record.manifest.id.clone();
    let revocations = std::collections::BTreeMap::from([(
        id.clone(),
        super::error_codes::FINGERPRINT_CHANGED.to_string(),
    )]);

    let mut records = vec![record];
    let reminder = super::registry_state::revoke_fingerprints(&mut records, &revocations);

    assert!(reminder);
    assert!(!records[0].enabled);
    assert!(!records[0].trusted);
    assert!(records[0].trusted_at.is_none());
    assert_eq!(records[0].status, super::types::ExtensionStatus::Error);
    assert_eq!(
        records[0].last_error.as_deref(),
        Some(super::error_codes::FINGERPRINT_CHANGED)
    );
}

#[test]
fn startup_keeps_a_persisted_fingerprint_revocation_visible_and_closed() {
    let (_directory, mut record) = source_tree();
    record.enabled = false;
    record.trusted = false;
    record.status = super::types::ExtensionStatus::Error;
    record.last_error = Some(super::error_codes::FINGERPRINT_FAILED.to_string());

    let reset = super::registry_state::reset_hosted_runtime(vec![record]);

    assert!(!reset[0].enabled);
    assert!(!reset[0].trusted);
    assert_eq!(reset[0].status, super::types::ExtensionStatus::Error);
    assert_eq!(
        reset[0].last_error.as_deref(),
        Some(super::error_codes::FINGERPRINT_FAILED)
    );
}

#[test]
fn fingerprint_limits_only_the_files_that_can_affect_execution() {
    let (directory, record) = source_tree();
    let ignored = directory.path().join("asset.bin");
    std::fs::File::create(&ignored)
        .unwrap()
        .set_len(super::types::FINGERPRINT_MAX_FILE_BYTES as u64 + 1)
        .unwrap();
    assert!(fingerprint::calculate(&record).is_ok());

    let source = directory.path().join("oversized.ts");
    std::fs::File::create(&source)
        .unwrap()
        .set_len(super::types::FINGERPRINT_MAX_FILE_BYTES as u64 + 1)
        .unwrap();
    assert!(super::managed_tree::validate(directory.path()).is_ok());
    assert!(fingerprint::calculate(&record).is_err());
}

#[test]
fn fingerprint_enforces_file_count_and_depth_limits() {
    let (directory, record) = source_tree();
    for index in 0..super::types::FINGERPRINT_MAX_FILES {
        std::fs::write(directory.path().join(format!("source-{index}.ts")), "").unwrap();
    }
    assert!(fingerprint::calculate(&record).is_err());

    let (directory, record) = source_tree();
    let mut nested = directory.path().to_path_buf();
    for _ in 0..=super::types::FINGERPRINT_MAX_DEPTH {
        nested = nested.join("nested");
        std::fs::create_dir(&nested).unwrap();
    }
    std::fs::write(nested.join("deep.ts"), "").unwrap();
    assert!(fingerprint::calculate(&record).is_err());
}

#[test]
fn fingerprint_enforces_the_total_source_budget() {
    let (directory, record) = source_tree();
    let full_files =
        super::types::FINGERPRINT_MAX_TOTAL_BYTES / super::types::FINGERPRINT_MAX_FILE_BYTES;
    for index in 0..full_files {
        std::fs::File::create(directory.path().join(format!("large-{index}.ts")))
            .unwrap()
            .set_len(super::types::FINGERPRINT_MAX_FILE_BYTES as u64)
            .unwrap();
    }

    assert!(fingerprint::calculate(&record).is_err());
}

#[tokio::test]
async fn changed_extension_never_reaches_a_host_load_request() {
    let (changed_directory, mut changed) = source_tree();
    changed.manifest.id = "com.example.changed-load".to_string();
    changed.enabled = true;
    changed.trusted = true;
    changed.fingerprint = Some(fingerprint::calculate(&changed).unwrap());
    std::fs::write(changed_directory.path().join("helper.ts"), "changed\n").unwrap();

    let (_healthy_directory, mut healthy) = source_tree();
    healthy.manifest.id = "com.example.healthy-load".to_string();
    healthy.enabled = true;
    healthy.trusted = true;
    healthy.fingerprint = Some(fingerprint::calculate(&healthy).unwrap());

    let verified = fingerprint::verify_records(vec![changed, healthy]);
    let plan = super::runtime_plan::records(verified.eligible);
    let host_root = tempfile::tempdir().unwrap();
    let loaded_ids = host_root.path().join("loaded.jsonl");
    let script = host_root.path().join("host.mjs");
    let loaded_literal = serde_json::to_string(loaded_ids.to_str().unwrap()).unwrap();
    std::fs::write(
        &script,
        format!(
            r#"import fs from "node:fs";
import readline from "node:readline";
const output = {loaded_literal};
readline.createInterface({{ input: process.stdin }}).on("line", (line) => {{
  const message = JSON.parse(line);
  if (message.method === "host.load") fs.appendFileSync(output, message.params.extension.id + "\n");
  const result = message.method === "host.load"
    ? {{ id: message.params.extension.id, contributions: {{ tools: [], events: [] }} }}
    : {{}};
  process.stdout.write(JSON.stringify({{ jsonrpc: "2.0", id: message.id, result }}) + "\n");
}});"#
        ),
    )
    .unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work = super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    let process = super::host_process::HostProcess::spawn(
        &super::host_paths::HostPaths {
            node: which::which("node").unwrap().canonicalize().unwrap(),
            script,
            directory: host_root.path().to_path_buf(),
        },
        &work,
    )
    .await
    .unwrap();
    let specs = plan
        .third_party
        .into_iter()
        .filter_map(|record| super::runtime_plan::specification(record, host_root.path()))
        .collect::<Vec<_>>();
    let mut responses = Vec::new();

    super::runtime_channel_sync::load_specs(&process, &specs, &mut responses)
        .await
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(&loaded_ids).unwrap(),
        "com.example.healthy-load\n"
    );
    assert!(
        process
            .kill(super::runtime_lifecycle::new_stop_deadline())
            .await
    );
}

#[test]
fn runtime_revokes_and_clears_permissions_before_loading_or_building_specs() {
    let source = include_str!("runtime_sync.rs");
    let verified = source.find("verify_records(records)").unwrap();
    let revoked = source
        .find("revoke_fingerprints(&verified.revocations)")
        .unwrap();
    let cleared = source.find("permission_gate::clear_extension").unwrap();
    let loading = source.find("registry_sync::mark_loading").unwrap();
    let planned = source.find("runtime_plan::records").unwrap();

    assert!(verified < revoked);
    assert!(revoked < cleared);
    assert!(cleared < loading);
    assert!(loading < planned);
}

#[cfg(unix)]
#[test]
fn symlinks_are_rejected_even_when_their_name_would_be_excluded() {
    use std::os::unix::fs::symlink;
    let (directory, record) = source_tree();
    symlink("/tmp", directory.path().join("node_modules")).unwrap();

    assert!(fingerprint::calculate(&record).is_err());
}
