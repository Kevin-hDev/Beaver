use super::types::{
    ExtensionContributions, ExtensionKind, ExtensionRecord, ExtensionStatus, MAX_EXTENSIONS,
    MAX_USER_EXTENSIONS,
};
use std::sync::{LazyLock, Mutex, RwLock};

static RECORDS: LazyLock<RwLock<Vec<ExtensionRecord>>> = LazyLock::new(|| RwLock::new(Vec::new()));
static MUTATIONS: Mutex<()> = Mutex::new(());

pub fn init() -> Result<(), String> {
    let stored = super::storage::load()?;
    let records = super::builtin::merge(super::registry_state::reset_hosted_runtime(stored))?;
    super::validation::records(&records)?;
    super::storage::save(&records)?;
    if super::managed_cleanup::unreferenced(&records).is_err() {
        super::operation_error::report(
            super::operation_error::Operation::Cleanup,
            super::OperationFailure::CleanupFailed,
        );
    }
    replace(records)
}

pub fn list() -> Result<Vec<ExtensionRecord>, String> {
    RECORDS
        .read()
        .map(|records| records.clone())
        .map_err(|_| "Registre d'extensions indisponible.".to_string())
}

pub(super) fn refresh_index() -> Result<(), String> {
    super::registry_index::rebuild(&list()?)
}

pub fn find(id: &str) -> Result<ExtensionRecord, String> {
    super::validation::identifier(id)?;
    list()?
        .into_iter()
        .find(|record| record.manifest.id == id)
        .ok_or_else(|| "Extension introuvable.".to_string())
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

pub fn remove(id: &str) -> Result<(), String> {
    super::validation::identifier(id)?;
    mutate(|records| {
        let index = records
            .iter()
            .position(|record| record.manifest.id == id)
            .ok_or_else(|| "Extension introuvable.".to_string())?;
        if records[index].kind == ExtensionKind::Builtin {
            return Err("Un plugin Beaver ne peut pas être supprimé.".to_string());
        }
        records.remove(index);
        Ok(())
    })
}

pub fn replace_user(
    expected: &ExtensionRecord,
    replacement: ExtensionRecord,
) -> Result<(), String> {
    let id = expected.manifest.id.as_str();
    super::validation::identifier(id)?;
    super::validation::records(std::slice::from_ref(&replacement))?;
    if replacement.kind != ExtensionKind::Local || replacement.manifest.id != id {
        return Err("Mise à jour d'extension invalide.".to_string());
    }
    mutate(|records| {
        let record = records
            .iter_mut()
            .find(|record| record.manifest.id == id)
            .ok_or_else(|| "Extension introuvable.".to_string())?;
        if record.kind == ExtensionKind::Builtin {
            return Err("Un plugin Beaver ne peut pas être remplacé.".to_string());
        }
        if record.source != expected.source || record.origin != expected.origin {
            return Err("L'extension a changé pendant sa mise à jour.".to_string());
        }
        *record = replacement;
        Ok(())
    })
}

pub fn set_enabled(id: &str, enabled: bool, trust_confirmed: bool) -> Result<(), String> {
    update(id, |record| {
        if enabled && record.kind != ExtensionKind::Builtin && !record.trusted && !trust_confirmed {
            return Err("Confirmation d'activation requise.".to_string());
        }
        if enabled && trust_confirmed {
            record.trusted = true;
        }
        record.enabled = enabled;
        record.status = ExtensionStatus::Inactive;
        record.last_error = None;
        if enabled {
            record.last_activated_at = Some(chrono::Utc::now().to_rfc3339());
        } else {
            record.contributions = ExtensionContributions::default();
        }
        Ok(())
    })
}

pub fn set_show_in_chat(id: &str, show: bool) -> Result<(), String> {
    update(id, |record| {
        record.show_in_chat = show;
        Ok(())
    })
}

pub fn disable_hosted_extensions() -> Result<(), String> {
    mutate(|records| {
        disable_hosted_records(records);
        Ok(())
    })
}

pub(super) fn disable_hosted_records(records: &mut [ExtensionRecord]) {
    for record in records {
        if record.kind != ExtensionKind::External {
            record.enabled = false;
            record.status = ExtensionStatus::Inactive;
            record.last_error = None;
            record.contributions = ExtensionContributions::default();
        }
    }
}

pub fn enabled_hosted() -> Result<Vec<ExtensionRecord>, String> {
    Ok(list()?
        .into_iter()
        .filter(|record| record.kind != ExtensionKind::External && record.enabled && record.trusted)
        .collect())
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
            .ok_or_else(|| "Extension introuvable.".to_string())?;
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
    let _guard = MUTATIONS.lock().map_err(|_| E::storage())?;
    let mut candidate = list().map_err(|_| E::storage())?;
    operation(&mut candidate)?;
    super::storage::save(&candidate).map_err(|_| E::storage())?;
    replace(candidate).map_err(|_| E::storage())
}

fn replace(records: Vec<ExtensionRecord>) -> Result<(), String> {
    let mut state = RECORDS
        .write()
        .map_err(|_| "Registre d'extensions indisponible.".to_string())?;
    super::registry_index::rebuild(&records)?;
    *state = records;
    Ok(())
}
