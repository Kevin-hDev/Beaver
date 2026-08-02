use std::path::PathBuf;

#[derive(Default)]
pub(super) struct CacheOverrides {
    pub cargo_home: Option<PathBuf>,
    pub npm_cache: Option<PathBuf>,
    pub pip_cache: Option<PathBuf>,
    pub uv_cache: Option<PathBuf>,
    pub go_build: Option<PathBuf>,
    pub yarn_cache: Option<PathBuf>,
    pub go_modules: Option<PathBuf>,
    pub go_path: Option<PathBuf>,
    pub xdg_cache: Option<PathBuf>,
}

impl CacheOverrides {
    pub fn from_env() -> Self {
        Self {
            cargo_home: env_path("CARGO_HOME"),
            npm_cache: env_path("npm_config_cache"),
            pip_cache: env_path("PIP_CACHE_DIR"),
            uv_cache: env_path("UV_CACHE_DIR"),
            go_build: env_path("GOCACHE"),
            yarn_cache: env_path("YARN_CACHE_FOLDER"),
            go_modules: env_path("GOMODCACHE"),
            go_path: std::env::var_os("GOPATH")
                .and_then(|value| std::env::split_paths(&value).find(|path| path.is_absolute())),
            xdg_cache: env_path("XDG_CACHE_HOME"),
        }
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}
