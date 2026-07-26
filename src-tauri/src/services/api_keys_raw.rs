const MAX_RAW_VALUE_LEN: usize = 8192;
const MAX_RAW_KEY_LEN: usize = 160;
const MAX_VAULT_ENTRIES: usize = 500;
const MAX_BATCH_ENTRIES: usize = 8;
const RAW_PREFIX: &str = "raw:";

pub fn has_key(provider_id: &str) -> bool {
    let state = STATE.lock().ok();
    state
        .as_ref()
        .and_then(|current| current.as_ref())
        .map(|current| current.keys.contains_key(provider_id))
        .unwrap_or(false)
}

pub fn list_configured() -> Vec<String> {
    configured_from_state()
}

pub fn set_raw(key: &str, value: &str) -> Result<(), String> {
    set_raw_batch(&[(key, value)])
}

pub fn set_raw_batch(entries: &[(&str, &str)]) -> Result<(), String> {
    validate_raw_batch(entries)?;
    let prefixed_keys: Vec<String> = entries
        .iter()
        .map(|(key, _)| format!("{RAW_PREFIX}{key}"))
        .collect();
    transaction(|candidate| {
        for (key, (_, value)) in prefixed_keys.iter().zip(entries) {
            candidate.insert(key.clone(), (*value).to_string());
        }
        Ok(())
    })
}

fn validate_raw_batch(entries: &[(&str, &str)]) -> Result<(), String> {
    if entries.is_empty() || entries.len() > MAX_BATCH_ENTRIES {
        return Err("lot de secrets invalide".to_string());
    }
    let mut unique = std::collections::HashSet::with_capacity(entries.len());
    for (key, value) in entries {
        if key.is_empty() || key.len() > MAX_RAW_KEY_LEN || !unique.insert(*key) {
            return Err("clé du coffre invalide".to_string());
        }
        if value.is_empty() || value.len() > MAX_RAW_VALUE_LEN {
            return Err("valeur du coffre invalide".to_string());
        }
    }
    Ok(())
}

pub fn get_raw(key: &str) -> Result<Zeroizing<String>, String> {
    let prefixed = format!("{RAW_PREFIX}{key}");
    let state = STATE.lock().map_err(|_| "erreur de stockage".to_string())?;
    let current = state
        .as_ref()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    current
        .keys
        .get(&prefixed)
        .cloned()
        .ok_or_else(|| "clé non trouvée".to_string())
}

pub fn get_or_create_random_raw(key: &str, byte_len: usize) -> Result<Zeroizing<Vec<u8>>, String> {
    use base64::Engine;
    use rand::RngCore;

    if key.is_empty() || key.len() > MAX_RAW_KEY_LEN || !(16..=64).contains(&byte_len) {
        return Err("clé du coffre invalide".to_string());
    }
    let prefixed = format!("{RAW_PREFIX}{key}");
    let mut state = STATE.lock().map_err(|_| "coffre indisponible".to_string())?;
    let current = state
        .as_mut()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    if let Some(encoded) = current.keys.get(&prefixed) {
        let decoded = Zeroizing::new(
            base64::engine::general_purpose::STANDARD
                .decode(encoded.as_bytes())
                .map_err(|_| "coffre invalide".to_string())?,
        );
        if decoded.len() != byte_len {
            return Err("coffre invalide".to_string());
        }
        return Ok(decoded);
    }

    let mut random = Zeroizing::new(vec![0_u8; byte_len]);
    rand::rngs::OsRng.fill_bytes(&mut random);
    let encoded = Zeroizing::new(base64::engine::general_purpose::STANDARD.encode(&*random));
    commit_candidate_with(
        current,
        |candidate| {
            candidate.insert(prefixed, encoded.to_string());
            Ok(())
        },
        vault::write_vault,
    )?;
    Ok(random)
}

pub fn delete_raw(key: &str) -> Result<(), String> {
    delete_raw_batch(&[key])
}

pub fn delete_raw_batch(keys: &[&str]) -> Result<(), String> {
    validate_raw_keys(keys)?;
    let prefixed: Vec<String> = keys.iter().map(|key| format!("{RAW_PREFIX}{key}")).collect();
    transaction(|candidate| {
        for key in &prefixed {
            candidate.remove(key);
        }
        Ok(())
    })
}

fn validate_raw_keys(keys: &[&str]) -> Result<(), String> {
    if keys.is_empty() || keys.len() > MAX_BATCH_ENTRIES {
        return Err("lot de secrets invalide".to_string());
    }
    let mut unique = std::collections::HashSet::with_capacity(keys.len());
    for key in keys {
        if key.is_empty() || key.len() > MAX_RAW_KEY_LEN || !unique.insert(*key) {
            return Err("clé du coffre invalide".to_string());
        }
    }
    Ok(())
}
