use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex as StdMutex, Weak};
use tokio::sync::Mutex;

const MAX_REPOSITORY_LOCKS: usize = 64;
type RepositoryLock = Arc<Mutex<()>>;
static LOCKS: LazyLock<StdMutex<HashMap<std::path::PathBuf, Weak<Mutex<()>>>>> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

pub fn for_repo(project_path: &Path) -> Result<RepositoryLock, String> {
    let canonical = project_path
        .canonicalize()
        .map_err(|_| "Projet Git indisponible".to_string())?;
    let mut locks = LOCKS
        .lock()
        .map_err(|_| "Verrou Git indisponible".to_string())?;
    locks.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = locks.get(&canonical).and_then(Weak::upgrade) {
        return Ok(lock);
    }
    if locks.len() >= MAX_REPOSITORY_LOCKS {
        return Err("Trop de projets Git actifs".into());
    }
    let lock = Arc::new(Mutex::new(()));
    locks.insert(canonical, Arc::downgrade(&lock));
    Ok(lock)
}

pub async fn acquire(project_path: &Path) -> Result<tokio::sync::OwnedMutexGuard<()>, String> {
    Ok(for_repo(project_path)?.lock_owned().await)
}
