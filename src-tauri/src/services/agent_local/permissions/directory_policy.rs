use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

const POLICY_ERROR: &str = "Politique d’accès indisponible.";

#[derive(Clone)]
struct Policy {
    stored_paths: Vec<String>,
    roots: Vec<PathBuf>,
}

static POLICY: OnceLock<RwLock<Result<Policy, ()>>> = OnceLock::new();

#[cfg(test)]
#[path = "directory_policy_test_support.rs"]
pub(crate) mod test_support;

pub(super) fn initialize() -> Result<(), String> {
    let state = load();
    let success = state.is_ok();
    let lock = POLICY.get_or_init(|| RwLock::new(Err(())));
    *lock.write().unwrap_or_else(|error| error.into_inner()) = state;
    success.then_some(()).ok_or_else(error)
}

pub(super) fn roots() -> Result<Vec<PathBuf>, String> {
    #[cfg(test)]
    if let Some(roots) = test_support::current_roots() {
        return Ok(roots);
    }
    if POLICY.get().is_none() {
        initialize()?;
    }
    let state = POLICY
        .get()
        .ok_or_else(error)?
        .read()
        .unwrap_or_else(|failure| failure.into_inner());
    state.as_ref().map(|policy| policy.roots.clone()).map_err(|_| error())
}

pub(super) fn cached_paths() -> Option<Vec<String>> {
    let state = POLICY.get()?.read().unwrap_or_else(|error| error.into_inner());
    state.as_ref().ok().map(|policy| policy.stored_paths.clone())
}

pub(super) fn replace(paths: Vec<String>) -> Result<Vec<PathBuf>, String> {
    let roots = super::directory_access::configured_roots_from_paths(paths.clone())?;
    crate::services::config::update_config(|config| {
        config.advanced.allowed_paths = paths.clone();
        Ok(())
    })?;
    let policy = Policy {
        stored_paths: paths,
        roots: roots.clone(),
    };
    let lock = POLICY.get_or_init(|| RwLock::new(Err(())));
    *lock.write().unwrap_or_else(|error| error.into_inner()) = Ok(policy);
    Ok(roots)
}

fn load() -> Result<Policy, ()> {
    let paths = crate::services::config::read_allowed_paths_strict().map_err(|_| ())?;
    let roots = super::directory_access::configured_roots_from_paths(paths.clone()).map_err(|_| ())?;
    Ok(Policy {
        stored_paths: paths,
        roots,
    })
}

fn error() -> String {
    POLICY_ERROR.to_string()
}
