use super::tool_cache_env::CacheOverrides;
use super::tool_cache_platform::{self, Platform};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub(super) enum CacheKind {
    CargoRegistry,
    CargoGit,
    Npm,
    Pip,
    Uv,
    GoBuild,
    Yarn,
    GoModules,
    GoSumDb,
}

const CACHE_KINDS: [CacheKind; 9] = [
    CacheKind::CargoRegistry,
    CacheKind::CargoGit,
    CacheKind::Npm,
    CacheKind::Pip,
    CacheKind::Uv,
    CacheKind::GoBuild,
    CacheKind::Yarn,
    CacheKind::GoModules,
    CacheKind::GoSumDb,
];

pub(super) const MAX_WRITE_DIRS: usize = CACHE_KINDS.len();

pub(super) fn collect(home: &Path, path_dirs: &[PathBuf], path_overflow: bool) -> Vec<PathBuf> {
    let platform = tool_cache_platform::current();
    let platform_cache = dirs::cache_dir();
    let selected = collect_for(
        platform,
        home,
        platform_cache.as_deref(),
        &CacheOverrides::from_env(),
    );
    debug_assert!(selected.len() <= MAX_WRITE_DIRS);
    let default_cache = match platform {
        #[cfg(any(test, target_os = "linux"))]
        Platform::Linux => tool_cache_platform::default_cache_base(platform, home),
        #[cfg(any(test, target_os = "macos"))]
        Platform::Macos => platform_cache
            .unwrap_or_else(|| tool_cache_platform::default_cache_base(platform, home)),
        #[cfg(any(test, windows))]
        Platform::Windows => platform_cache
            .unwrap_or_else(|| tool_cache_platform::default_cache_base(platform, home)),
    };
    let defaults = collect_for(
        platform,
        home,
        Some(&default_cache),
        &CacheOverrides::default(),
    );
    super::tool_cache_prepare::ensure_defaults(
        &CACHE_KINDS,
        &selected,
        &defaults,
        home,
        path_dirs,
        path_overflow,
    );
    selected
}

fn collect_for(
    platform: Platform,
    home: &Path,
    platform_cache: Option<&Path>,
    overrides: &CacheOverrides,
) -> Vec<PathBuf> {
    let fallback_cache = tool_cache_platform::default_cache_base(platform, home);
    let platform_cache = platform_cache.unwrap_or(&fallback_cache);
    let xdg_cache = overrides
        .xdg_cache
        .as_deref()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".cache"));
    CACHE_KINDS
        .iter()
        .map(|kind| match kind {
            CacheKind::CargoRegistry => cargo_home(home, overrides).join("registry"),
            CacheKind::CargoGit => cargo_home(home, overrides).join("git"),
            CacheKind::Npm => npm_root(platform, home, platform_cache, overrides).join("_cacache"),
            CacheKind::Pip => overrides
                .pip_cache
                .clone()
                .unwrap_or_else(|| pip_cache(platform, platform_cache)),
            CacheKind::Uv => overrides.uv_cache.clone().unwrap_or_else(|| {
                if tool_cache_platform::is_windows(platform) {
                    platform_cache.join("uv/cache")
                } else {
                    xdg_cache.join("uv")
                }
            }),
            CacheKind::GoBuild => overrides
                .go_build
                .clone()
                .unwrap_or_else(|| platform_cache.join("go-build")),
            CacheKind::Yarn => overrides
                .yarn_cache
                .clone()
                .unwrap_or_else(|| yarn_cache(platform, platform_cache)),
            CacheKind::GoModules => overrides
                .go_modules
                .clone()
                .unwrap_or_else(|| go_path(home, overrides).join("pkg/mod")),
            CacheKind::GoSumDb => go_path(home, overrides).join("pkg/sumdb"),
        })
        .collect()
}

fn go_path(home: &Path, overrides: &CacheOverrides) -> PathBuf {
    overrides
        .go_path
        .clone()
        .unwrap_or_else(|| home.join("go"))
}

fn cargo_home(home: &Path, overrides: &CacheOverrides) -> PathBuf {
    overrides
        .cargo_home
        .clone()
        .unwrap_or_else(|| home.join(".cargo"))
}

fn npm_root(
    platform: Platform,
    home: &Path,
    platform_cache: &Path,
    overrides: &CacheOverrides,
) -> PathBuf {
    overrides.npm_cache.clone().unwrap_or_else(|| {
        if tool_cache_platform::is_windows(platform) {
            platform_cache.join("npm-cache")
        } else {
            home.join(".npm")
        }
    })
}

fn pip_cache(platform: Platform, platform_cache: &Path) -> PathBuf {
    if tool_cache_platform::is_windows(platform) {
        platform_cache.join("pip/Cache")
    } else {
        platform_cache.join("pip")
    }
}

fn yarn_cache(platform: Platform, platform_cache: &Path) -> PathBuf {
    match platform {
        #[cfg(any(test, target_os = "macos"))]
        Platform::Macos => platform_cache.join("Yarn"),
        #[cfg(any(test, target_os = "linux"))]
        Platform::Linux => platform_cache.join("yarn"),
        #[cfg(any(test, windows))]
        Platform::Windows => platform_cache.join("Yarn/Cache"),
    }
}

#[cfg(test)]
#[path = "tool_cache_roots_tests.rs"]
mod tests;
