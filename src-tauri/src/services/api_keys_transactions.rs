fn commit_candidate_with<Mutate, Persist>(
    state: &mut VaultState,
    mutate: Mutate,
    persist: Persist,
) -> Result<(), String>
where
    Mutate: FnOnce(&mut HashMap<String, String>) -> Result<(), String>,
    Persist: FnOnce(&[u8], &HashMap<String, String>) -> Result<(), String>,
{
    let mut candidate = ZeroizingMap(
        state
            .keys
            .iter()
            .map(|(key, value)| (key.clone(), value.as_str().to_string()))
            .collect(),
    );
    mutate(&mut candidate.0)?;
    validate_vault_candidate(&candidate.0)?;
    persist(&state.master_key, &candidate.0)?;
    state.keys = candidate
        .0
        .drain()
        .map(|(key, value)| (key, Zeroizing::new(value)))
        .collect();
    Ok(())
}

pub(crate) fn stage_raw_entries(
    candidate: &mut HashMap<String, String>,
    entries: &[(&str, &str)],
) -> Result<(), String> {
    validate_raw_entries(entries)?;
    let physical_keys: Vec<String> = entries
        .iter()
        .map(|(key, _)| prefixed_raw_key(key))
        .collect::<Result<_, _>>()?;
    let additions = physical_keys
        .iter()
        .filter(|key| !candidate.contains_key(*key))
        .count();
    let final_len = candidate
        .len()
        .checked_add(additions)
        .ok_or_else(|| "limite du coffre atteinte".to_string())?;
    if final_len > MAX_VAULT_ENTRIES {
        return Err("limite du coffre atteinte".to_string());
    }
    for (physical, (_, value)) in physical_keys.into_iter().zip(entries) {
        candidate.insert(physical, (*value).to_string());
    }
    Ok(())
}

fn validate_vault_candidate(candidate: &HashMap<String, String>) -> Result<(), String> {
    if candidate.len() > MAX_VAULT_ENTRIES {
        return Err("limite du coffre atteinte".to_string());
    }
    for (key, value) in candidate {
        if let Some(logical) = key.strip_prefix(RAW_PREFIX) {
            validate_raw_entry(logical, value)?;
        }
    }
    Ok(())
}

fn validate_raw_entries(entries: &[(&str, &str)]) -> Result<(), String> {
    if entries.is_empty() || entries.len() > MAX_BATCH_ENTRIES {
        return Err("lot de secrets invalide".to_string());
    }
    let mut unique = std::collections::HashSet::with_capacity(entries.len());
    for (key, value) in entries {
        if !unique.insert(*key) {
            return Err("clé du coffre invalide".to_string());
        }
        validate_raw_entry(key, value)?;
    }
    Ok(())
}

fn validate_raw_entry(key: &str, value: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > MAX_RAW_KEY_LEN {
        return Err("clé du coffre invalide".to_string());
    }
    if value.is_empty() || value.len() > MAX_RAW_VALUE_LEN {
        return Err("valeur du coffre invalide".to_string());
    }
    Ok(())
}

fn transaction<Mutate>(mutate: Mutate) -> Result<(), String>
where
    Mutate: FnOnce(&mut HashMap<String, String>) -> Result<(), String>,
{
    let mut state = STATE
        .lock()
        .map_err(|_| "coffre indisponible".to_string())?;
    let current = state
        .as_mut()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    commit_candidate_with(current, mutate, vault::write_vault)
}
