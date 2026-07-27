use super::protocol::{HostExtensionSpec, SyncResult};
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionDiagnostic, MAX_EXTENSIONS,
};
use std::collections::{HashMap, HashSet};

pub struct BuildSpecs {
    pub specs: Vec<HostExtensionSpec>,
    pub enabled_ids: HashSet<String>,
    pub diagnostics: Vec<ExtensionDiagnostic>,
}

pub struct ApplyResult {
    pub active: usize,
    pub diagnostics: Vec<ExtensionDiagnostic>,
}

pub fn build_specs(records: Vec<super::types::ExtensionRecord>) -> Result<BuildSpecs, String> {
    super::registry_sync::mark_enabled_loading()?;
    let enabled_ids = records
        .iter()
        .take(MAX_EXTENSIONS)
        .map(|record| record.manifest.id.clone())
        .collect();
    let mut specs = Vec::new();
    let mut diagnostics = Vec::new();
    for record in records.into_iter().take(MAX_EXTENSIONS) {
        let extension_id = record.manifest.id.clone();
        if let Some(specification) = build_spec(record) {
            specs.push(specification);
        } else {
            diagnostics.push(ExtensionDiagnostic {
                extension_id,
                stage: "import".to_string(),
                code: "entry_unavailable".to_string(),
                file: None,
                line: None,
                column: None,
            });
        }
    }
    Ok(BuildSpecs {
        specs,
        enabled_ids,
        diagnostics,
    })
}

pub fn apply(response: SyncResult, build: &BuildSpecs) -> Result<ApplyResult, String> {
    if response.extensions.len() > MAX_EXTENSIONS {
        return Err("Réponse de l'hôte d'extensions invalide.".to_string());
    }
    let requested: HashMap<&str, &HostExtensionSpec> = build
        .specs
        .iter()
        .map(|spec| (spec.id.as_str(), spec))
        .collect();
    let mut received = HashSet::new();
    let mut successful = HashMap::new();
    let mut diagnostics = build.diagnostics.clone();
    for loaded in response.extensions.into_iter().take(MAX_EXTENSIONS) {
        if loaded
            .error
            .as_deref()
            .is_some_and(|error| error != "load_failed")
            || (loaded.diagnostic.is_some() && loaded.error.is_none())
        {
            return Err("Réponse de l'hôte d'extensions invalide.".to_string());
        }
        let Some(spec) = requested.get(loaded.id.as_str()) else {
            return Err("Réponse de l'hôte d'extensions invalide.".to_string());
        };
        if !received.insert(loaded.id.clone()) {
            return Err("Réponse de l'hôte d'extensions invalide.".to_string());
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
            diagnostics.push(runtime_diagnostic(&loaded.id, "load_failed"));
        }
        let Some(contributions) = loaded.contributions.filter(|_| loaded.error.is_none()) else {
            continue;
        };
        if accepts_contributions(spec, &contributions) {
            successful.insert(loaded.id, contributions);
        } else {
            diagnostics.push(ExtensionDiagnostic {
                extension_id: loaded.id,
                stage: "register".to_string(),
                code: "advanced_required".to_string(),
                file: None,
                line: None,
                column: None,
            });
        }
    }
    for missing in requested
        .keys()
        .filter(|extension_id| !received.contains(**extension_id))
    {
        diagnostics.push(runtime_diagnostic(missing, "host_missing_response"));
    }
    let active = super::registry_sync::apply_results(&build.enabled_ids, successful)?;
    Ok(ApplyResult {
        active,
        diagnostics,
    })
}

fn runtime_diagnostic(extension_id: &str, code: &str) -> ExtensionDiagnostic {
    ExtensionDiagnostic {
        extension_id: extension_id.to_string(),
        stage: "import".to_string(),
        code: code.to_string(),
        file: None,
        line: None,
        column: None,
    }
}

fn build_spec(record: super::types::ExtensionRecord) -> Option<HostExtensionSpec> {
    let main = super::manifest::resolve_record_entry(&record).ok()?;
    let main_path = main.to_str()?;
    Some(HostExtensionSpec {
        id: record.manifest.id.clone(),
        main_path: main_path.to_string(),
        manifest: record.manifest,
    })
}

fn accepts_contributions(spec: &HostExtensionSpec, contributions: &ExtensionContributions) -> bool {
    spec.manifest.api_level == ExtensionApiLevel::Advanced
        || !contributions.tools.iter().any(|tool| tool.replaces_core)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::extensions::types::{ExtensionManifest, ExtensionTool};
    use serde_json::json;

    #[test]
    fn stable_extensions_cannot_replace_core_tools() {
        let spec = HostExtensionSpec {
            id: "com.example.stable".to_string(),
            main_path: "/tmp/index.ts".to_string(),
            manifest: ExtensionManifest {
                id: "com.example.stable".to_string(),
                name: "Stable".to_string(),
                version: "1.0.0".to_string(),
                beaver_api: "1".to_string(),
                runtime: "node".to_string(),
                main: Some("index.ts".to_string()),
                ui: None,
                access: "full".to_string(),
                api_level: ExtensionApiLevel::Stable,
                author: None,
                homepage: None,
                description: None,
            },
        };
        let contributions = ExtensionContributions {
            tools: vec![ExtensionTool {
                name: "web_search".to_string(),
                description: "Replacement".to_string(),
                parameters: json!({"type": "object"}),
                replaces_core: true,
            }],
            events: Vec::new(),
        };

        assert!(!accepts_contributions(&spec, &contributions));
    }
}
