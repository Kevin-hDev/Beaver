use super::types::{
    ExtensionContributions, ExtensionKind, ExtensionRecord, ExtensionStatus, ExtensionTool,
    MAX_EXTENSIONS, MAX_TOOLS,
};
use std::sync::{LazyLock, Mutex, RwLock};

static RECORDS: LazyLock<RwLock<Vec<ExtensionRecord>>> =
    LazyLock::new(|| RwLock::new(super::builtin::records()));
static MUTATIONS: Mutex<()> = Mutex::new(());

pub fn init() -> Result<(), String> {
    let stored = super::storage::load()?;
    let records = super::builtin::merge(reset_local_runtime(stored));
    super::validation::records(&records)?;
    super::storage::save(&records)?;
    replace(records)
}

pub fn list() -> Result<Vec<ExtensionRecord>, String> {
    RECORDS
        .read()
        .map(|records| records.clone())
        .map_err(|_| "Registre d'extensions indisponible.".to_string())
}

pub fn add_local(record: ExtensionRecord) -> Result<(), String> {
    super::validation::records(std::slice::from_ref(&record))?;
    mutate(|records| {
        if records.len() >= MAX_EXTENSIONS {
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

pub fn set_enabled(id: &str, enabled: bool) -> Result<(), String> {
    update(id, |record| {
        record.enabled = enabled;
        record.status = if enabled && record.kind == ExtensionKind::Builtin {
            ExtensionStatus::Active
        } else {
            ExtensionStatus::Inactive
        };
        record.last_error = None;
        if enabled {
            record.last_activated_at = Some(chrono::Utc::now().to_rfc3339());
        } else if record.kind != ExtensionKind::Builtin {
            record.contributions = ExtensionContributions::default();
        }
    })
}

pub fn set_show_in_chat(id: &str, show: bool) -> Result<(), String> {
    update(id, |record| record.show_in_chat = show)
}

pub fn disable_user_extensions() -> Result<(), String> {
    mutate(|records| {
        for record in records {
            if record.kind != ExtensionKind::Builtin {
                record.enabled = false;
                record.status = ExtensionStatus::Inactive;
                record.last_error = None;
                record.contributions = ExtensionContributions::default();
            }
        }
        Ok(())
    })
}

pub fn mark_loading(id: &str) -> Result<(), String> {
    update(id, |record| {
        record.status = ExtensionStatus::Loading;
        record.last_error = None;
    })
}

pub fn apply_loaded(id: &str, contributions: ExtensionContributions) -> Result<(), String> {
    super::validation::contributions(&contributions.tools, &contributions.events)?;
    let current_total = list()?
        .iter()
        .filter(|record| record.manifest.id != id && record.enabled)
        .map(|record| record.contributions.tools.len())
        .sum::<usize>();
    if current_total.saturating_add(contributions.tools.len()) > MAX_TOOLS {
        return Err("Nombre maximal d'outils d'extension atteint.".to_string());
    }
    update(id, |record| {
        record.contributions = contributions;
        record.status = ExtensionStatus::Active;
        record.last_error = None;
    })
}

pub fn mark_error(id: &str) {
    let _ = update(id, |record| {
        record.status = ExtensionStatus::Error;
        record.last_error = Some("Impossible de charger cette extension.".to_string());
        record.contributions = ExtensionContributions::default();
    });
}

pub fn enabled_local() -> Result<Vec<ExtensionRecord>, String> {
    Ok(list()?
        .into_iter()
        .filter(|record| record.kind == ExtensionKind::Local && record.enabled)
        .collect())
}

pub fn dynamic_tools() -> Vec<ExtensionTool> {
    list()
        .unwrap_or_default()
        .into_iter()
        .filter(|record| {
            record.kind == ExtensionKind::Local
                && record.enabled
                && record.status == ExtensionStatus::Active
        })
        .flat_map(|record| record.contributions.tools)
        .collect()
}

pub fn is_dynamic_tool(tool_name: &str) -> bool {
    dynamic_tools().iter().any(|tool| tool.name == tool_name)
}

fn update(id: &str, update: impl FnOnce(&mut ExtensionRecord)) -> Result<(), String> {
    super::validation::identifier(id)?;
    mutate(|records| {
        let record = records
            .iter_mut()
            .find(|record| record.manifest.id == id)
            .ok_or_else(|| "Extension introuvable.".to_string())?;
        update(record);
        Ok(())
    })
}

fn mutate(
    operation: impl FnOnce(&mut Vec<ExtensionRecord>) -> Result<(), String>,
) -> Result<(), String> {
    let _guard = MUTATIONS
        .lock()
        .map_err(|_| "Registre d'extensions indisponible.".to_string())?;
    let mut candidate = list()?;
    operation(&mut candidate)?;
    super::storage::save(&candidate)?;
    replace(candidate)
}

fn replace(records: Vec<ExtensionRecord>) -> Result<(), String> {
    let mut state = RECORDS
        .write()
        .map_err(|_| "Registre d'extensions indisponible.".to_string())?;
    *state = records;
    Ok(())
}

fn reset_local_runtime(mut records: Vec<ExtensionRecord>) -> Vec<ExtensionRecord> {
    for record in &mut records {
        if record.kind == ExtensionKind::Local {
            record.status = ExtensionStatus::Inactive;
            record.last_error = None;
            record.contributions = ExtensionContributions::default();
        }
    }
    records
}
