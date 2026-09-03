use super::types::{
    ExtensionContributions, ExtensionKind, ExtensionRecord, ExtensionStatus, MAX_EXTENSIONS,
    MAX_USER_EXTENSIONS,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub(super) static MUTATIONS: Mutex<()> = Mutex::new(());

pub fn init() -> Result<(), String> {
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

pub fn list() -> Result<Vec<ExtensionRecord>, String> {
    super::registry_memory::records()
}

pub(super) fn refresh_index() -> Result<(), String> {
    super::registry_index::rebuild(&list()?)
}

pub fn find(id: &str) -> Result<ExtensionRecord, String> {
    super::validation::identifier(id)?;
    list()?
        .into_iter()
        .find(|record| record.manifest.id == id)
        .ok_or_else(|| super::error_codes::NOT_FOUND.to_string())
}

pub fn add_local(record: ExtensionRecord) -> Result<(), String> {
    super::validation::records(std::slice::from_ref(&record))?;
    mutate(|records| {
        let user_extensions = records
            .iter()
            .filter(|item| item.kind != ExtensionKind::Builtin)
            .count();
        if user_extensions >= MAX_USER_EXTENSIONS || records.len() >= MAX_EXTENSIONS {
            return Err("Nombre maximal d'extensions atteint.".to_string());
        }
        if records
            .iter()
            .any(|item| item.manifest.id == record.manifest.id)
        {
            return Err("Cette extension est déjà enregistrée.".to_string());
        }
        records.push(record);
        Ok(())
    })
}

pub fn remove(id: &str) -> Result<bool, String> {
    super::validation::identifier(id)?;
    let mut reminder = false;
    mutate(|records| {
        let index = records
            .iter()
            .position(|record| record.manifest.id == id)
            .ok_or_else(|| super::error_codes::NOT_FOUND.to_string())?;
        if records[index].kind == ExtensionKind::Builtin {
            return Err("Un plugin Beaver ne peut pas être supprimé.".to_string());
        }
        reminder = records.remove(index).sensitive_access_granted;
        Ok(())
    })?;
    Ok(reminder)
}

pub fn replace_user(
    expected: &ExtensionRecord,
    mut replacement: ExtensionRecord,
) -> Result<bool, String> {
    let id = expected.manifest.id.as_str();
    super::validation::identifier(id)?;
    super::validation::records(std::slice::from_ref(&replacement))?;
    if replacement.kind != ExtensionKind::Local || replacement.manifest.id != id {
        return Err("Mise à jour d'extension invalide.".to_string());
    }
    let mut reminder = false;
    mutate(|records| {
        let record = records
            .iter_mut()
            .find(|record| record.manifest.id == id)
            .ok_or_else(|| super::error_codes::NOT_FOUND.to_string())?;
        if record.kind == ExtensionKind::Builtin {
            return Err("Un plugin Beaver ne peut pas être remplacé.".to_string());
        }
        if record.source != expected.source || record.origin != expected.origin {
            return Err("L'extension a changé pendant sa mise à jour.".to_string());
        }
        reminder = super::installer_record::carry_sensitive_access(record, &mut replacement);
        *record = replacement;
        Ok(())
    })?;
    Ok(reminder)
}

pub(super) fn replace_ui_records(
    replacements: Vec<(ExtensionRecord, ExtensionRecord)>,
) -> Result<(), String> {
    for (_, replacement) in &replacements {
        super::validation::records(std::slice::from_ref(replacement))?;
    }
    mutate(|records| {
        for (expected, replacement) in replacements {
            let current = records
                .iter_mut()
                .find(|record| record.manifest.id == expected.manifest.id)
                .ok_or_else(|| super::error_codes::NOT_FOUND.to_string())?;
            if current.kind != ExtensionKind::Local
                || current.source != expected.source
                || current.origin != expected.origin
            {
                return Err("L'extension a changé pendant sa recharge.".to_string());
            }
            *current = replacement;
        }
        Ok(())
    })
}

pub async fn set_enabled(id: &str, enabled: bool, trust_confirmed: bool) -> Result<bool, String> {
    let mut reminder = false;
    update(id, |record| {
        if enabled && record.kind != ExtensionKind::Builtin && !record.trusted && !trust_confirmed {
            return Err(super::error_codes::ACTIVATION_CONFIRMATION_REQUIRED.to_string());
        }
        let activated_at = chrono::Utc::now().to_rfc3339();
        if enabled && trust_confirmed && record.kind == ExtensionKind::Local {
            super::registry_state::approve_local(record, &activated_at)?;
        }
        let preserve_revocation =
            !enabled && super::registry_state::preserve_revocation_on_disable(record);
        record.enabled = enabled;
        if !preserve_revocation {
            record.status = ExtensionStatus::Inactive;
            record.last_error = None;
        }
        if enabled {
            record.last_activated_at = Some(activated_at);
        } else {
            reminder = record.sensitive_access_granted;
            record.contributions = ExtensionContributions::default();
        }
        Ok(())
    })?;
    if !enabled {
        crate::services::agent_local::permission_gate::clear_extension(id).await;
    }
    Ok(reminder)
}

pub fn set_show_in_chat(id: &str, show: bool) -> Result<(), String> {
    update(id, |record| {
        record.show_in_chat = show;
        Ok(())
    })
}

pub(super) fn revoke_fingerprints(revocations: &BTreeMap<String, String>) -> Result<bool, String> {
    if revocations.is_empty() {
        return Ok(false);
    }
    let mut reminder = false;
    mutate(|records| {
        reminder = super::registry_state::revoke_fingerprints(records, revocations);
        Ok::<(), String>(())
    })?;
    Ok(reminder)
}

fn update(
    id: &str,
    update: impl FnOnce(&mut ExtensionRecord) -> Result<(), String>,
) -> Result<(), String> {
    super::validation::identifier(id)?;
    mutate(|records| {
        let record = records
            .iter_mut()
            .find(|record| record.manifest.id == id)
            .ok_or_else(|| super::error_codes::NOT_FOUND.to_string())?;
        update(record)?;
        Ok(())
    })
}

pub(super) fn mutate<E>(
    operation: impl FnOnce(&mut Vec<ExtensionRecord>) -> Result<(), E>,
) -> Result<(), E>
where
    E: super::registry_mutation_error::MutationError,
{
    mutate_state(|records, _| operation(records))
}

pub(super) fn mutate_state<E>(
    operation: impl FnOnce(&mut Vec<ExtensionRecord>, &mut Option<Vec<String>>) -> Result<(), E>,
) -> Result<(), E>
where
    E: super::registry_mutation_error::MutationError,
{
    let _guard = MUTATIONS.lock().map_err(|_| E::storage())?;
    let mut candidate = super::registry_memory::snapshot().map_err(|_| E::storage())?;
    operation(&mut candidate.records, &mut candidate.recovery_snapshot)?;
    super::storage::save(&candidate.records, &candidate.recovery_snapshot)
        .map_err(|_| E::storage())?;
    super::registry_memory::replace(candidate.records, candidate.recovery_snapshot)
        .map_err(|_| E::storage())
}
