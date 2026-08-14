#[cfg(not(windows))]
use super::constants::MAX_OLLAMA_ENV_TOTAL_UNIX_BYTES;
#[cfg(windows)]
use super::constants::MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16;
use super::constants::{
    MAX_OLLAMA_ENV_ENTRIES, MAX_OLLAMA_ENV_KEY_UNITS, MAX_OLLAMA_ENV_VALUE_UNITS,
};
use super::error::OllamaErrorCode;
use std::ffi::{OsStr, OsString};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrozenEnvironment {
    entries: Vec<(OsString, OsString)>,
}

impl FrozenEnvironment {
    #[cfg(test)]
    pub(crate) fn from_entries(entries: Vec<(OsString, OsString)>) -> Self {
        assert!(entries.len() <= MAX_OLLAMA_ENV_ENTRIES);
        Self { entries }
    }
    #[allow(dead_code)]
    pub(crate) fn entries(&self) -> &[(OsString, OsString)] {
        &self.entries
    }
    #[allow(dead_code)]
    pub(crate) fn get(&self, key: &str) -> Option<&str> {
        self.entries.iter().find_map(|(name, value)| {
            same_key(name, OsStr::new(key))
                .then(|| value.to_str())
                .flatten()
        })
    }
    #[allow(dead_code)]
    pub(crate) fn count(&self, key: &str) -> usize {
        self.entries
            .iter()
            .filter(|(name, _)| same_key(name, OsStr::new(key)))
            .count()
    }
    pub(crate) fn value(&self, key: &str) -> Option<&OsStr> {
        self.entries
            .iter()
            .find_map(|(name, value)| same_key(name, OsStr::new(key)).then_some(value.as_os_str()))
    }
}

pub(crate) fn freeze(
    inherited: Vec<(OsString, OsString)>,
    overrides: Vec<(OsString, OsString)>,
) -> Result<FrozenEnvironment, OllamaErrorCode> {
    if inherited.len() > MAX_OLLAMA_ENV_ENTRIES {
        return Err(OllamaErrorCode::OllamaInternal);
    }
    validate_unique(&inherited)?;
    validate_entries(&inherited)?;
    validate_unique(&overrides)?;
    validate_entries(&overrides)?;
    if overrides
        .iter()
        .any(|(key, _)| same_key(key, OsStr::new("OLLAMA_HOST")))
    {
        return Err(OllamaErrorCode::OllamaInternal);
    }
    let mut entries = inherited
        .into_iter()
        .filter(|(key, _)| {
            !same_key(key, OsStr::new("OLLAMA_HOST"))
                && !overrides
                    .iter()
                    .any(|(override_key, _)| same_key(key, override_key))
        })
        .collect::<Vec<_>>();
    entries.extend(overrides);
    if entries.len() > MAX_OLLAMA_ENV_ENTRIES {
        return Err(OllamaErrorCode::OllamaInternal);
    }
    validate_unique(&entries)?;
    validate_entries(&entries)?;
    Ok(FrozenEnvironment { entries })
}

pub(crate) fn collect_bounded<I>(entries: I) -> Result<Vec<(OsString, OsString)>, OllamaErrorCode>
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    let mut collected = Vec::new();
    for entry in entries {
        if collected.len() >= MAX_OLLAMA_ENV_ENTRIES {
            return Err(OllamaErrorCode::OllamaInternal);
        }
        collected.push(entry);
    }
    Ok(collected)
}

pub(crate) fn freeze_from_snapshot(
    snapshot: FrozenEnvironment,
    overrides: Vec<(OsString, OsString)>,
) -> Result<FrozenEnvironment, OllamaErrorCode> {
    freeze(snapshot.entries().to_vec(), overrides)
}

fn validate_unique(entries: &[(OsString, OsString)]) -> Result<(), OllamaErrorCode> {
    for (index, (key, _)) in entries.iter().enumerate() {
        if entries[..index]
            .iter()
            .any(|(previous, _)| same_key(previous, key))
        {
            return Err(OllamaErrorCode::OllamaInternal);
        }
    }
    Ok(())
}

fn same_key(left: &OsStr, right: &OsStr) -> bool {
    #[cfg(windows)]
    {
        left.to_string_lossy().to_lowercase() == right.to_string_lossy().to_lowercase()
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}

fn validate_entries(entries: &[(OsString, OsString)]) -> Result<(), OllamaErrorCode> {
    let mut total = 0usize;
    for (key, value) in entries {
        if key.is_empty()
            || key.to_string_lossy().contains('=')
            || key.to_string_lossy().contains('\0')
            || value.to_string_lossy().contains('\0')
        {
            return Err(OllamaErrorCode::OllamaInternal);
        }
        let key_units = units(key);
        let value_units = units(value);
        if key_units > MAX_OLLAMA_ENV_KEY_UNITS || value_units > MAX_OLLAMA_ENV_VALUE_UNITS {
            return Err(OllamaErrorCode::OllamaInternal);
        }
        total = total
            .checked_add(key_units + value_units + 2)
            .ok_or(OllamaErrorCode::OllamaInternal)?;
    }
    #[cfg(windows)]
    if total > MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16 {
        return Err(OllamaErrorCode::OllamaInternal);
    }
    #[cfg(not(windows))]
    if total > MAX_OLLAMA_ENV_TOTAL_UNIX_BYTES {
        return Err(OllamaErrorCode::OllamaInternal);
    }
    Ok(())
}

fn units(value: &OsStr) -> usize {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        value.as_bytes().len()
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        value.encode_wide().count()
    }
    #[cfg(not(any(unix, windows)))]
    {
        value.to_string_lossy().len()
    }
}
