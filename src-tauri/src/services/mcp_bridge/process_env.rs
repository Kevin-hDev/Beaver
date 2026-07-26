const PASSTHROUGH_ENV: &[&str] = &[
    "PATH",
    "HOME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "USER",
    "SHELL",
    "XDG_DATA_HOME",
    "XDG_CACHE_HOME",
    "XDG_CONFIG_HOME",
];

const APP_CACHE_ENV: &[(&str, &str)] = &[
    ("NPM_CONFIG_CACHE", "npm-cache"),
    ("DENO_DIR", "deno"),
    ("UV_CACHE_DIR", "uv-cache"),
];

pub fn safe_env() -> Result<Vec<(String, String)>, String> {
    let mut env: Vec<(String, String)> = PASSTHROUGH_ENV
        .iter()
        .filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v)))
        .collect();

    let runtime_dir = crate::services::paths::data_dir().join("mcp-runtime");
    std::fs::create_dir_all(&runtime_dir)
        .map_err(|_| "cache runtime MCP indisponible".to_string())?;

    for (key, child_dir) in APP_CACHE_ENV {
        let dir = runtime_dir.join(child_dir);
        std::fs::create_dir_all(&dir).map_err(|_| "cache runtime MCP indisponible".to_string())?;
        env.push((key.to_string(), dir.to_string_lossy().into_owned()));
    }

    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_user_package_resolution_env() {
        let env = safe_env().unwrap();
        let keys: Vec<&str> = env.iter().map(|(key, _)| key.as_str()).collect();
        assert!(!keys.contains(&"NODE_PATH"));
        assert!(!keys.contains(&"NPM_CONFIG_PREFIX"));
    }

    #[test]
    fn uses_app_owned_runtime_caches() {
        let env = safe_env().unwrap();
        let data_dir = crate::services::paths::data_dir();
        for key in ["NPM_CONFIG_CACHE", "DENO_DIR", "UV_CACHE_DIR"] {
            let value = env
                .iter()
                .find(|(env_key, _)| env_key == key)
                .map(|(_, value)| value)
                .expect("cache env manquant");
            assert!(std::path::Path::new(value).starts_with(&data_dir));
        }
    }
}
