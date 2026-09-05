use super::types::{
    ExtensionDiagnostic, DIAGNOSTIC_ADVANCED_REQUIRED, DIAGNOSTIC_LOAD_FAILED,
    HOST_LOAD_STAGE_REGISTER, MAX_RUNTIME_DIAGNOSTICS,
};

fn diagnostic(extension_id: &str, code: &str) -> ExtensionDiagnostic {
    ExtensionDiagnostic {
        extension_id: extension_id.to_string(),
        stage: HOST_LOAD_STAGE_REGISTER.to_string(),
        code: code.to_string(),
        occurred_at: "2026-09-03T10:00:00Z".to_string(),
        file: None,
        line: None,
        column: None,
    }
}

#[test]
fn ui_diagnostics_are_validated_then_projected_once_per_extension() {
    let mut diagnostics = Vec::new();
    let first =
        super::runtime_sync_apply::ui_diagnostic("com.example.ui", "ui_contribution_invalid")
            .unwrap();
    let second =
        super::runtime_sync_apply::ui_diagnostic("com.example.ui", "ui_limit_exceeded").unwrap();
    super::runtime_sync_apply::push_ui_diagnostic_once(&mut diagnostics, first).unwrap();
    super::runtime_sync_apply::push_ui_diagnostic_once(&mut diagnostics, second).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert!(chrono::DateTime::parse_from_rfc3339(&diagnostics[0].occurred_at).is_ok());
    assert!(super::runtime_sync_apply::ui_diagnostic("com.example.ui", "unknown").is_err());
}

#[test]
fn runtime_diagnostic_projection_refuses_growth_past_its_bound() {
    let mut diagnostics = (0..MAX_RUNTIME_DIAGNOSTICS)
        .map(|index| diagnostic(&format!("com.example.{index}"), "load_failed"))
        .collect::<Vec<_>>();
    assert!(super::runtime_sync_apply::push_diagnostic(
        &mut diagnostics,
        diagnostic("com.example.overflow", "load_failed"),
    )
    .is_err());
    assert_eq!(diagnostics.len(), MAX_RUNTIME_DIAGNOSTICS);
}

#[test]
fn contribution_validation_failures_keep_their_distinct_generic_diagnostics() {
    use super::runtime_sync_contributions::ValidationError;

    assert_eq!(
        super::runtime_sync_apply::contribution_diagnostic_code(ValidationError::AdvancedRequired),
        DIAGNOSTIC_ADVANCED_REQUIRED,
    );
    assert_eq!(
        super::runtime_sync_apply::contribution_diagnostic_code(
            ValidationError::InvalidContribution
        ),
        DIAGNOSTIC_LOAD_FAILED,
    );
}
