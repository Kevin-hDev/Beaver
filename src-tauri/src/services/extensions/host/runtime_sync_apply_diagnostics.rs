pub(super) fn contribution_diagnostic_code(
    error: super::runtime_sync_contributions::ValidationError,
) -> &'static str {
    match error {
        super::runtime_sync_contributions::ValidationError::AdvancedRequired => {
            super::types::DIAGNOSTIC_ADVANCED_REQUIRED
        }
        // Une forme Hôte invalide n'est pas une demande d'autorisation avancée.
        super::runtime_sync_contributions::ValidationError::InvalidContribution => {
            super::types::DIAGNOSTIC_LOAD_FAILED
        }
    }
}
