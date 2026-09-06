pub(super) fn initialize() -> Result<(), String> {
    let _guard = super::registry::MUTATIONS
        .lock()
        .map_err(|_| super::error_codes::REGISTRY_UNAVAILABLE.to_string())?;
    super::registry_index::mark_unavailable(super::error_codes::REGISTRY_UNAVAILABLE)?;
    let result = load_and_publish();
    if let Err(error) = &result {
        super::registry_index::mark_unavailable(error)?;
    }
    result
}

fn load_and_publish() -> Result<(), String> {
    let loaded = super::storage::load()?;
    let format = loaded.format;
    let recovery_snapshot = loaded.recovery_snapshot;
    let records = super::builtin::merge(super::registry_state::reset_hosted_runtime(
        loaded.extensions,
    ))?;
    super::validation::records(&records)?;
    super::storage::save(&records, &recovery_snapshot)?;
    if super::managed_cleanup::unreferenced(&records).is_err() {
        super::operation_error::report(
            super::operation_error::Operation::Cleanup,
            super::OperationFailure::CleanupFailed,
        );
    }
    if super::ui_artifact_store::unreferenced(&records).is_err() {
        super::operation_error::report(
            super::operation_error::Operation::Cleanup,
            super::OperationFailure::CleanupFailed,
        );
    }
    super::registry_memory::replace(records, recovery_snapshot)?;
    super::storage::finish_successful_startup(&super::storage::path(), format)
}

pub(super) fn refuse(error: &str) -> Result<(), String> {
    let _guard = super::registry::MUTATIONS
        .lock()
        .map_err(|_| super::error_codes::REGISTRY_UNAVAILABLE.to_string())?;
    super::registry_index::mark_unavailable(error)
}

#[cfg(test)]
pub(crate) fn initialize_test_registry() {
    static INITIALIZED: std::sync::Once = std::sync::Once::new();
    INITIALIZED.call_once(|| {
        let _guard = super::registry::MUTATIONS.lock().unwrap();
        // Integration fixtures need a published catalog, but startup cleanup must
        // never remove staged directories owned by other concurrently running tests.
        super::registry_memory::replace(Vec::new(), None).unwrap();
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn uninitialized_registry_refuses_mutation_before_running_the_operation() {
        const CHILD: &str = "BEAVER_TEST_UNINITIALIZED_REGISTRY";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "services::extensions::registry_startup::tests::uninitialized_registry_refuses_mutation_before_running_the_operation", "--nocapture"])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let result = super::super::registry::mutate::<String>(|_| {
            panic!("an unavailable registry must never execute a mutation")
        });
        assert_eq!(
            result,
            Err(super::super::error_codes::REGISTRY_UNAVAILABLE.to_string())
        );
        super::super::registry_memory::replace(Vec::new(), None).unwrap();
        assert!(super::super::registry_availability().is_ok());
        let result = super::super::registry::refresh_index_with(|| Err("rebuild failed".into()));
        assert_eq!(
            result,
            Err(super::super::error_codes::REGISTRY_UNAVAILABLE.to_string())
        );
        assert!(super::super::registry_catalog().is_err());
        assert!(super::super::registry::mutate::<String>(|_| panic!("closed registry")).is_err());
        super::super::registry_memory::replace(Vec::new(), None).unwrap();
        assert!(super::super::registry_catalog().is_ok());
    }
}
