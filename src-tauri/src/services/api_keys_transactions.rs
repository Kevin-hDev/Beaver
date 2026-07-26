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
    if candidate.0.len() > MAX_VAULT_ENTRIES {
        return Err("limite du coffre atteinte".to_string());
    }
    persist(&state.master_key, &candidate.0)?;
    state.keys = candidate
        .0
        .drain()
        .map(|(key, value)| (key, Zeroizing::new(value)))
        .collect();
    Ok(())
}

fn transaction<Mutate>(mutate: Mutate) -> Result<(), String>
where
    Mutate: FnOnce(&mut HashMap<String, String>) -> Result<(), String>,
{
    let mut state = STATE.lock().map_err(|_| "coffre indisponible".to_string())?;
    let current = state
        .as_mut()
        .ok_or_else(|| "coffre indisponible".to_string())?;
    commit_candidate_with(current, mutate, vault::write_vault)
}
