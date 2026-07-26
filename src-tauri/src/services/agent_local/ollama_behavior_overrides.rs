use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const MAX_PROMPTS: usize = 128;
const MAX_STORE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Default, Serialize, Deserialize)]
struct BehaviorStore {
    prompts: BTreeMap<String, String>,
}

pub fn get(model: &str) -> Option<String> {
    super::model_customizations::validate_model_name(model).ok()?;
    let store = store_cache().lock().ok()?;
    store.prompts.get(model).cloned()
}

pub fn set(model: &str, prompt: &str) -> Result<(), String> {
    super::model_customizations::validate_model_name(model)?;
    let normalized = super::ollama_modelfile_system::normalize_prompt(prompt)?;
    let mut store = store_cache()
        .lock()
        .map_err(|_| "ollama-system-store-write".to_string())?;
    persist_update(&mut store, model, normalized, write_store)
}

fn persist_update<F>(
    store: &mut BehaviorStore,
    model: &str,
    prompt: Option<String>,
    persist: F,
) -> Result<(), String>
where
    F: FnOnce(&BehaviorStore) -> Result<(), String>,
{
    let mut candidate = store.clone();
    update_store(&mut candidate, model, prompt)?;
    persist(&candidate)?;
    *store = candidate;
    Ok(())
}

fn update_store(
    store: &mut BehaviorStore,
    model: &str,
    prompt: Option<String>,
) -> Result<(), String> {
    match prompt {
        Some(value) => {
            if store.prompts.len() >= MAX_PROMPTS && !store.prompts.contains_key(model) {
                return Err("ollama-system-prompt-limit".into());
            }
            store.prompts.insert(model.to_string(), value);
        }
        None => {
            store.prompts.remove(model);
        }
    }
    Ok(())
}

fn read_store() -> BehaviorStore {
    let path = store_path();
    if std::fs::metadata(&path)
        .map(|metadata| metadata.len() > MAX_STORE_BYTES)
        .unwrap_or(false)
    {
        return BehaviorStore::default();
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return BehaviorStore::default();
    };
    let Ok(store) = serde_json::from_str::<BehaviorStore>(&content) else {
        return BehaviorStore::default();
    };
    sanitize_store(store)
}

fn sanitize_store(mut store: BehaviorStore) -> BehaviorStore {
    store.prompts.retain(|model, prompt| {
        super::model_customizations::validate_model_name(model).is_ok()
            && super::ollama_modelfile_system::normalize_prompt(prompt).is_ok_and(|p| p.is_some())
    });
    store.prompts = store.prompts.into_iter().take(MAX_PROMPTS).collect();
    store
}

fn write_store(store: &BehaviorStore) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(store)
        .map_err(|_| "ollama-system-store-write".to_string())?;
    if data.len() as u64 > MAX_STORE_BYTES {
        return Err("ollama-system-prompt-limit".into());
    }
    crate::services::private_store::atomic_write(&store_path(), &data)
        .map_err(|_| "ollama-system-store-write".to_string())
}

fn store_path() -> PathBuf {
    crate::services::paths::data_dir().join("ollama-system-prompts.json")
}

fn store_cache() -> &'static Mutex<BehaviorStore> {
    static CACHE: OnceLock<Mutex<BehaviorStore>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(read_store()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_model_names_before_storage() {
        assert!(set("../invalid", "custom").is_err());
    }

    #[test]
    fn deserialization_is_bounded_and_filters_invalid_entries() {
        let mut prompts = BTreeMap::new();
        prompts.insert("../invalid".to_string(), "custom".to_string());
        prompts.insert("valid:model".to_string(), "".to_string());
        let store = BehaviorStore { prompts };
        let content = serde_json::to_string(&store).unwrap();
        let decoded: BehaviorStore = serde_json::from_str(&content).unwrap();
        let decoded = sanitize_store(decoded);
        assert!(decoded.prompts.is_empty());
    }

    #[test]
    fn empty_prompt_restores_default_behavior_for_only_that_model() {
        let mut store = BehaviorStore::default();
        update_store(&mut store, "first:model", Some("custom".into())).unwrap();
        update_store(&mut store, "second:model", Some("other".into())).unwrap();
        update_store(&mut store, "first:model", None).unwrap();

        assert!(!store.prompts.contains_key("first:model"));
        assert_eq!(store.prompts.get("second:model").map(String::as_str), Some("other"));
    }

    #[test]
    fn failed_persistence_does_not_modify_the_memory_cache() {
        let mut store = BehaviorStore::default();
        update_store(&mut store, "model:old", Some("old".into())).unwrap();

        let result = persist_update(&mut store, "model:new", Some("new".into()), |_| {
            Err("write failed".to_string())
        });

        assert!(result.is_err());
        assert_eq!(store.prompts.len(), 1);
        assert_eq!(store.prompts.get("model:old").map(String::as_str), Some("old"));
    }
}
