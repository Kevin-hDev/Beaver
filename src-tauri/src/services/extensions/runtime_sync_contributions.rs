use super::host_identity::HostIdentity;
use super::protocol::HostExtensionSpec;
use super::types::ExtensionContributions;
use super::ui_types::UiCatalogEntry;

pub(super) struct ValidatedContributions {
    pub(super) core: ExtensionContributions,
    pub(super) ui: Vec<UiCatalogEntry>,
    pub(super) ui_diagnostic: Option<String>,
}

pub(super) fn validate(
    identity: &HostIdentity,
    extension_id: &str,
    specification: &HostExtensionSpec,
    mut contributions: ExtensionContributions,
) -> Result<ValidatedContributions, ()> {
    if !super::runtime_sync::accepts_contributions(specification, &contributions) {
        return Err(());
    }
    let raw_ui = std::mem::take(&mut contributions.ui);
    let (ui, ui_diagnostic) = match super::ui_validation::catalog(
        identity,
        extension_id,
        &specification.manifest.api_level,
        specification.manifest.ui.as_ref(),
        raw_ui,
    ) {
        Ok(entries) => (entries, None),
        Err(code) => (Vec::new(), Some(code)),
    };
    Ok(ValidatedContributions {
        core: contributions,
        ui,
        ui_diagnostic,
    })
}
