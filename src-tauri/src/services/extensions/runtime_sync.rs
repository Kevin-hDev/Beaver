use super::protocol::{HostExtensionSpec, LoadResult};
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionDiagnostic, DIAGNOSTIC_ADVANCED_REQUIRED,
    DIAGNOSTIC_ENTRY_UNAVAILABLE, DIAGNOSTIC_HOST_MISSING_RESPONSE, DIAGNOSTIC_LOAD_FAILED,
    HOST_LOAD_STAGE_IMPORT, HOST_LOAD_STAGE_REGISTER, MAX_EXTENSIONS, RUNTIME_DIAGNOSTIC_CODES,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

pub struct BuildSpecs {
    pub official_specs: Vec<HostExtensionSpec>,
    pub third_party_specs: BTreeMap<String, HostExtensionSpec>,
    pub enabled_ids: HashSet<String>,
    pub diagnostics: Vec<ExtensionDiagnostic>,
    pub failures: BTreeMap<String, String>,
    pub sensitive_access_reminder: bool,
}

pub struct ApplyResult {
    pub active: usize,
    pub diagnostics: Vec<ExtensionDiagnostic>,
    pub completed_ids: HashSet<String>,
}

pub(super) use super::runtime_recovery_preflight::{filter_for_recovery, RecoveryPreflight};

pub async fn build_specs(
    records: Vec<super::types::ExtensionRecord>,
    host_directory: &Path,
    recovery: &RecoveryPreflight,
) -> Result<BuildSpecs, String> {
    if matches!(recovery, RecoveryPreflight::Invalid) {
        // Sans identité authentifiable, on conserve le registre et on ne journalise
        // que le code générique, jamais le contenu ni le chemin du marqueur.
        ::log::warn!(
            "[extensions] {}",
            super::error_codes::RECOVERY_MARKER_INVALID
        );
    }
    if let RecoveryPreflight::Interrupted(extension_id) = recovery {
        super::registry_interruption::mark_interrupted(extension_id)?;
    }
    let recovered = filter_for_recovery(records, recovery);
    let verified = super::fingerprint::verify_records(recovered);
    let sensitive_access_reminder = super::registry::revoke_fingerprints(&verified.revocations)?;
    for extension_id in verified.revocations.keys() {
        crate::services::agent_local::permission_gate::clear_extension(extension_id).await;
    }
    let enabled_ids = verified
        .eligible
        .iter()
        .filter(|record| record.enabled && record.trusted)
        .take(MAX_EXTENSIONS)
        .map(|record| record.manifest.id.clone())
        .collect();
    super::registry_sync::mark_loading(&enabled_ids)?;
    let plan = super::runtime_plan::records(verified.eligible);
    let mut official_specs = Vec::new();
    let mut third_party_specs = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for record in plan.official.into_iter().take(MAX_EXTENSIONS) {
        let extension_id = record.manifest.id.clone();
        if let Some(specification) = super::runtime_plan::specification(record, host_directory) {
            official_specs.push(specification);
        } else {
            diagnostics.push(runtime_diagnostic(
                &extension_id,
                HOST_LOAD_STAGE_IMPORT,
                DIAGNOSTIC_ENTRY_UNAVAILABLE,
            ));
        }
    }
    for record in plan.third_party.into_iter().take(MAX_EXTENSIONS) {
        let extension_id = record.manifest.id.clone();
        if let Some(specification) = super::runtime_plan::specification(record, host_directory) {
            third_party_specs.insert(extension_id, specification);
        } else {
            diagnostics.push(runtime_diagnostic(
                &extension_id,
                HOST_LOAD_STAGE_IMPORT,
                DIAGNOSTIC_ENTRY_UNAVAILABLE,
            ));
        }
    }
    Ok(BuildSpecs {
        official_specs,
        third_party_specs,
        enabled_ids,
        diagnostics,
        failures: plan.failures,
        sensitive_access_reminder,
    })
}

pub fn apply(responses: Vec<LoadResult>, build: &BuildSpecs) -> Result<ApplyResult, String> {
    if responses.len() > MAX_EXTENSIONS {
        return Err(super::error_codes::HOST_INCOMPATIBLE.to_string());
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
    for loaded in responses.into_iter().take(MAX_EXTENSIONS) {
        if loaded
            .error
            .as_deref()
            .is_some_and(|error| error != "load_failed")
            || (loaded.diagnostic.is_some() && loaded.error.is_none())
        {
            return Err(super::error_codes::HOST_INCOMPATIBLE.to_string());
        }
        let Some(spec) = requested.get(loaded.id.as_str()) else {
            return Err(super::error_codes::HOST_INCOMPATIBLE.to_string());
        };
        if !received.insert(loaded.id.clone()) {
            return Err(super::error_codes::HOST_INCOMPATIBLE.to_string());
        }
        let failed = loaded.error.is_some();
        let has_diagnostic = loaded.diagnostic.is_some();
        if let Some(diagnostic) = loaded.diagnostic {
            diagnostics.push(super::runtime_diagnostics::from_host(
                loaded.id.clone(),
                diagnostic,
            )?);
        }
        if failed && !has_diagnostic {
            diagnostics.push(runtime_diagnostic(
                &loaded.id,
                HOST_LOAD_STAGE_IMPORT,
                DIAGNOSTIC_LOAD_FAILED,
            ));
        }
        let Some(contributions) = loaded.contributions.filter(|_| loaded.error.is_none()) else {
            continue;
        };
        if accepts_contributions(spec, &contributions) {
            successful.insert(loaded.id, contributions);
        } else {
            diagnostics.push(runtime_diagnostic(
                &loaded.id,
                HOST_LOAD_STAGE_REGISTER,
                DIAGNOSTIC_ADVANCED_REQUIRED,
            ));
        }
    }
    for missing in requested
        .keys()
        .filter(|extension_id| !received.contains(**extension_id))
    {
        diagnostics.push(runtime_diagnostic(
            missing,
            HOST_LOAD_STAGE_IMPORT,
            DIAGNOSTIC_HOST_MISSING_RESPONSE,
        ));
    }
    let active =
        super::registry_sync::apply_results(&build.enabled_ids, successful, &build.failures)?;
    Ok(ApplyResult {
        active,
        diagnostics,
        completed_ids: received,
    })
}

fn runtime_diagnostic(
    extension_id: &str,
    stage: &'static str,
    code: &'static str,
) -> ExtensionDiagnostic {
    debug_assert!(RUNTIME_DIAGNOSTIC_CODES.contains(&code));
    ExtensionDiagnostic {
        extension_id: extension_id.to_string(),
        stage: stage.to_string(),
        code: code.to_string(),
        file: None,
        line: None,
        column: None,
    }
}

pub(super) fn accepts_contributions(
    spec: &HostExtensionSpec,
    contributions: &ExtensionContributions,
) -> bool {
    spec.manifest.api_level == ExtensionApiLevel::Advanced
        || !contributions.tools.iter().any(|tool| tool.replaces_core)
}
