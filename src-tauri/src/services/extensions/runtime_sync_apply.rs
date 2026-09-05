use super::host_identity::HostIdentity;
use super::protocol::{AttributedLoadResult, HostExtensionSpec, LoadResult};
use super::runtime_sync::{ApplyResult, BuildSpecs};
use super::types::{
    ExtensionDiagnostic, DIAGNOSTIC_ADVANCED_REQUIRED, DIAGNOSTIC_HOST_MISSING_RESPONSE,
    DIAGNOSTIC_LOAD_FAILED, HOST_LOAD_STAGE_IMPORT, HOST_LOAD_STAGE_REGISTER, MAX_EXTENSIONS,
    MAX_RUNTIME_DIAGNOSTICS,
};
use std::collections::{HashMap, HashSet};

pub fn apply(
    responses: Vec<AttributedLoadResult>,
    build: &BuildSpecs,
) -> Result<ApplyResult, String> {
    if responses.len() > MAX_EXTENSIONS {
        return Err(incompatible());
    }
    let requested: HashMap<&str, &HostExtensionSpec> = build
        .official_specs
        .iter()
        .chain(build.third_party_specs.values())
        .map(|spec| (spec.id.as_str(), spec))
        .collect();
    let mut received = HashSet::new();
    let mut successful = HashMap::new();
    let mut diagnostics = build.diagnostics.clone();
    let mut ui_updates = Vec::with_capacity(responses.len());
    for response in responses.into_iter().take(MAX_EXTENSIONS) {
        let loaded = response.loaded;
        let spec = requested.get(loaded.id.as_str()).ok_or_else(incompatible)?;
        validate_attribution(&response.identity, response.generation, spec)?;
        if !received.insert(loaded.id.clone()) {
            return Err(incompatible());
        }
        validate_load_shape(&loaded)?;
        append_host_diagnostics(&loaded, &mut diagnostics)?;
        let mut ui_entries = Vec::new();
        if let Some(contributions) = loaded.contributions.filter(|_| loaded.error.is_none()) {
            match super::runtime_sync_contributions::validate(
                &response.identity,
                &loaded.id,
                spec,
                contributions,
            ) {
                Ok(validated) => {
                    ui_entries = validated.ui;
                    if let Some(code) = validated.ui_diagnostic {
                        push_ui_diagnostic_once(
                            &mut diagnostics,
                            ui_diagnostic(&loaded.id, &code)?,
                        )?;
                    }
                    successful.insert(loaded.id.clone(), validated.core);
                }
                Err(error) => push_diagnostic(
                    &mut diagnostics,
                    super::runtime_sync::runtime_diagnostic(
                        &loaded.id,
                        HOST_LOAD_STAGE_REGISTER,
                        contribution_diagnostic_code(error),
                    ),
                )?,
            }
        }
        ui_updates.push(super::ui_catalog::UiCatalogUpdate {
            identity: response.identity,
            generation: response.generation,
            extension_id: loaded.id,
            entries: ui_entries,
        });
    }
    append_missing(&requested, &received, &mut diagnostics)?;
    let active =
        super::registry_sync::apply_results(&build.enabled_ids, successful, &build.failures)?;
    Ok(ApplyResult {
        active,
        diagnostics,
        completed_ids: received,
        ui_updates,
    })
}

pub(super) fn contribution_diagnostic_code(
    error: super::runtime_sync_contributions::ValidationError,
) -> &'static str {
    match error {
        super::runtime_sync_contributions::ValidationError::AdvancedRequired => {
            DIAGNOSTIC_ADVANCED_REQUIRED
        }
        // La forme a été fournie par le processus Hôte : ne pas la présenter
        // comme une demande d'autorisation avancée lorsqu'elle est invalide.
        super::runtime_sync_contributions::ValidationError::InvalidContribution => {
            DIAGNOSTIC_LOAD_FAILED
        }
    }
}

fn validate_attribution(
    identity: &HostIdentity,
    generation: u64,
    spec: &HostExtensionSpec,
) -> Result<(), String> {
    let authorized = generation > 0
        && match identity {
            HostIdentity::Official => spec.id.starts_with("beaver."),
            HostIdentity::ThirdParty(id) => id == &spec.id,
        };
    authorized.then_some(()).ok_or_else(incompatible)
}

fn validate_load_shape(loaded: &LoadResult) -> Result<(), String> {
    if loaded
        .error
        .as_deref()
        .is_some_and(|error| error != "load_failed")
        || (loaded.diagnostic.is_some() && loaded.error.is_none())
    {
        return Err(incompatible());
    }
    Ok(())
}

fn append_host_diagnostics(
    loaded: &LoadResult,
    diagnostics: &mut Vec<ExtensionDiagnostic>,
) -> Result<(), String> {
    let max = super::ui_contract::MAX_CONTRIBUTIONS_PER_EXTENSION
        .checked_add(super::ui_contract::MAX_ACTIONS_PER_EXTENSION)
        .ok_or_else(incompatible)?;
    if loaded.ui_diagnostics.len() > max {
        return Err(incompatible());
    }
    // Tous les codes sont revalidés, mais une seule erreur UI est projetée par
    // extension afin de garder la collection et sa clé d'affichage stables.
    let mut first_ui_diagnostic = None;
    for diagnostic in &loaded.ui_diagnostics {
        let validated = ui_diagnostic(&loaded.id, &diagnostic.code)?;
        first_ui_diagnostic.get_or_insert(validated);
    }
    if let Some(diagnostic) = first_ui_diagnostic {
        push_ui_diagnostic_once(diagnostics, diagnostic)?;
    }
    if let Some(diagnostic) = loaded.diagnostic.clone() {
        push_diagnostic(
            diagnostics,
            super::runtime_diagnostics::from_host(loaded.id.clone(), diagnostic)?,
        )?;
    } else if loaded.error.is_some() {
        push_diagnostic(
            diagnostics,
            super::runtime_sync::runtime_diagnostic(
                &loaded.id,
                HOST_LOAD_STAGE_IMPORT,
                DIAGNOSTIC_LOAD_FAILED,
            ),
        )?;
    }
    Ok(())
}

fn append_missing(
    requested: &HashMap<&str, &HostExtensionSpec>,
    received: &HashSet<String>,
    diagnostics: &mut Vec<ExtensionDiagnostic>,
) -> Result<(), String> {
    for missing in requested
        .keys()
        .filter(|extension_id| !received.contains(**extension_id))
    {
        push_diagnostic(
            diagnostics,
            super::runtime_sync::runtime_diagnostic(
                missing,
                HOST_LOAD_STAGE_IMPORT,
                DIAGNOSTIC_HOST_MISSING_RESPONSE,
            ),
        )?;
    }
    Ok(())
}

pub(super) fn push_diagnostic(
    diagnostics: &mut Vec<ExtensionDiagnostic>,
    diagnostic: ExtensionDiagnostic,
) -> Result<(), String> {
    if diagnostics.len() >= MAX_RUNTIME_DIAGNOSTICS {
        return Err(incompatible());
    }
    diagnostics.push(diagnostic);
    Ok(())
}

pub(super) fn push_ui_diagnostic_once(
    diagnostics: &mut Vec<ExtensionDiagnostic>,
    diagnostic: ExtensionDiagnostic,
) -> Result<(), String> {
    if diagnostics.iter().any(|existing| {
        existing.extension_id == diagnostic.extension_id
            && super::ui_contract::UI_DIAGNOSTIC_CODES.contains(&existing.code.as_str())
    }) {
        return Ok(());
    }
    push_diagnostic(diagnostics, diagnostic)
}

pub(super) fn ui_diagnostic(extension_id: &str, code: &str) -> Result<ExtensionDiagnostic, String> {
    if !super::ui_contract::UI_DIAGNOSTIC_CODES.contains(&code) {
        return Err(incompatible());
    }
    Ok(ExtensionDiagnostic {
        extension_id: extension_id.to_string(),
        stage: HOST_LOAD_STAGE_REGISTER.to_string(),
        code: code.to_string(),
        occurred_at: super::diagnostic_time::now(),
        file: None,
        line: None,
        column: None,
    })
}

fn incompatible() -> String {
    super::error_codes::HOST_INCOMPATIBLE.to_string()
}
