const FORBIDDEN_ENV_KEYS: &[&str] = &[
    "PATH",
    "HOME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "USER",
    "SHELL",
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "DYLD_INSERT_LIBRARIES",
    "NODE_OPTIONS",
    "NODE_PATH",
    "DENO_DIR",
    "NPM_CONFIG_CACHE",
    "NPM_CONFIG_PREFIX",
    "UV_CACHE_DIR",
    "UV_TOOL_BIN_DIR",
    "UV_TOOL_DIR",
    "PYTHONHOME",
    "PYTHONPATH",
    "XDG_DATA_HOME",
    "XDG_CACHE_HOME",
    "XDG_CONFIG_HOME",
];
const MAX_ENV_KEYS: usize = 10;

pub fn validated_env_keys(keys: Option<&[String]>) -> Result<Vec<String>, String> {
    let keys = keys.unwrap_or_default();
    if keys.len() > MAX_ENV_KEYS {
        return Err("trop de variables d'environnement MCP".to_string());
    }
    for key in keys {
        validate_env_key(key)?;
    }
    Ok(keys.to_vec())
}

pub fn validate_env_key(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > 64 {
        return Err("identifiant invalide".to_string());
    }
    if !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err("identifiant invalide".to_string());
    }
    if FORBIDDEN_ENV_KEYS
        .iter()
        .any(|forbidden| forbidden.eq_ignore_ascii_case(key))
    {
        return Err("variable d'environnement MCP non autorisée".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_package_resolution_env_keys() {
        for key in [
            "NODE_PATH",
            "NPM_CONFIG_PREFIX",
            "UV_CACHE_DIR",
            "PYTHONPATH",
        ] {
            assert!(validate_env_key(key).is_err(), "{key} doit être refusé");
        }
    }

    #[test]
    fn accepts_connector_token_key() {
        assert!(validate_env_key("HF_TOKEN").is_ok());
    }
}
