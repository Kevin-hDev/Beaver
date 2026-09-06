use super::*;
use ts_rs::TS;
fn bindings() -> String {
    let declarations = [
        InstallRequest::decl(&ts_rs::Config::default()),
        InstallKind::decl(&ts_rs::Config::default()),
        InstallStatus::decl(&ts_rs::Config::default()),
        InstallPhase::decl(&ts_rs::Config::default()),
        QueueBlocker::decl(&ts_rs::Config::default()),
        InstallJobView::decl(&ts_rs::Config::default()),
        InstallJobsSnapshot::decl(&ts_rs::Config::default()),
    ];
    format!(
        "// Generated from Rust install_jobs/types.rs and limits.rs. Do not edit.\nexport const INSTALL_JOB_LIMITS = {{ active: {}, recent: {} }} as const;\n{}\n",
        super::limits::MAX_ACTIVE,
        super::limits::MAX_RECENT,
        declarations
            .into_iter()
            .map(|decl| format!("export {decl}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
#[test]
fn checked_in_typescript_matches_rust() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/extension-install-jobs.generated.ts");
    assert_eq!(std::fs::read_to_string(path).unwrap(), bindings());
}
#[test]
#[ignore = "refresh generated installation contract"]
fn export_typescript_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/extension-install-jobs.generated.ts");
    std::fs::write(path, bindings()).unwrap();
}
