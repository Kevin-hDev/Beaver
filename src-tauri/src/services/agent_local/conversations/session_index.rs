pub(crate) use super::session_index_io::write_index_to;
pub(crate) use super::session_index_meta::from_session as meta_from_session;
use super::session_index_io::{
    index_dir, index_fingerprint, index_path, read_index_from, write_index, IndexFingerprint,
};
use super::session_index_reconcile::reconcile as reconcile_index;
#[cfg(test)]
use super::session_index_reconcile::meta_drifted as index_meta_drifted;
use crate::services::agent_local::types_session::AgentSessionMeta;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Mutex;

// ponytail: one lock is enough for this bounded local index; split it only if profiling proves contention.
static INDEX_LOCK: Mutex<()> = Mutex::const_new(());
static INDEX_RECONCILE_FINGERPRINT: Mutex<Option<IndexFingerprint>> = Mutex::const_new(None);
static SESSION_SOURCE_REVISION: AtomicU64 = AtomicU64::new(0);
static INDEX_SOURCE_REVISION: AtomicU64 = AtomicU64::new(u64::MAX);
#[cfg(test)]
static FAIL_NEXT_UPSERT_SESSION: Mutex<Option<String>> = Mutex::const_new(None);

pub async fn read_index() -> Result<Vec<AgentSessionMeta>, String> {
    let _guard = INDEX_LOCK.lock().await;
    read_index_locked().await
}

async fn read_index_locked() -> Result<Vec<AgentSessionMeta>, String> {
    loop {
        let source_revision = SESSION_SOURCE_REVISION.load(Ordering::Acquire);
        let entries = read_index_once(&index_path(), source_revision).await?;
        if SESSION_SOURCE_REVISION.load(Ordering::Acquire) == source_revision {
            return Ok(entries);
        }
    }
}

async fn read_index_once(
    path: &Path,
    source_revision: u64,
) -> Result<Vec<AgentSessionMeta>, String> {
    let mut last_fingerprint = INDEX_RECONCILE_FINGERPRINT.lock().await;
    match read_index_from(path).await {
        Ok(entries) => {
            let fingerprint = index_fingerprint(path).await;
            if INDEX_SOURCE_REVISION.load(Ordering::Acquire) != source_revision {
                let dir = path
                    .parent()
                    .ok_or_else(|| "index indisponible".to_string())?;
                let entries = rebuild_index_from(dir).await?;
                *last_fingerprint = index_fingerprint(path).await;
                INDEX_SOURCE_REVISION.store(source_revision, Ordering::Release);
                Ok(entries)
            } else if last_fingerprint.as_ref() == fingerprint.as_ref() {
                Ok(entries)
            } else {
                let entries = reconcile_index(path, entries).await?;
                *last_fingerprint = index_fingerprint(path).await;
                INDEX_SOURCE_REVISION.store(source_revision, Ordering::Release);
                Ok(entries)
            }
        }
        Err(_) => {
            let dir = path
                .parent()
                .ok_or_else(|| "index indisponible".to_string())?;
            let entries = rebuild_index_from(dir).await?;
            *last_fingerprint = index_fingerprint(path).await;
            INDEX_SOURCE_REVISION.store(source_revision, Ordering::Release);
            Ok(entries)
        }
    }
}

pub async fn rebuild_index() -> Result<Vec<AgentSessionMeta>, String> {
    let _guard = INDEX_LOCK.lock().await;
    rebuild_global_index_locked().await
}

async fn rebuild_global_index_locked() -> Result<Vec<AgentSessionMeta>, String> {
    loop {
        let source_revision = SESSION_SOURCE_REVISION.load(Ordering::Acquire);
        let entries = rebuild_index_from(index_dir().as_path()).await?;
        if SESSION_SOURCE_REVISION.load(Ordering::Acquire) != source_revision {
            continue;
        }
        refresh_reconcile_state(&index_path(), source_revision).await;
        return Ok(entries);
    }
}

pub async fn rebuild_index_from(dir: &Path) -> Result<Vec<AgentSessionMeta>, String> {
    let mut entries = Vec::new();
    let mut evicted = 0_usize;
    let mut read_dir = match tokio::fs::read_dir(dir).await {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            write_index_to(dir, &entries).await?;
            return Ok(entries);
        }
        Err(error) => return Err(error.to_string()),
    };
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("index.json") {
            continue;
        }
        #[cfg(test)]
        test_support::record_document_read();
        if let Ok(session) = super::session_store_document::read_from_path(path).await {
            entries.push(meta_from_session(&session));
            if entries.len() >= super::session_index_io::MAX_REBUILD_BUFFER_ENTRIES {
                let bounded = super::session_index_io::retain_recent(entries);
                entries = bounded.0;
                evicted = evicted.saturating_add(bounded.1);
            }
        }
    }
    let bounded = super::session_index_io::retain_recent(entries);
    entries = bounded.0;
    evicted = evicted.saturating_add(bounded.1);
    if evicted > 0 {
        ::log::warn!("[session-index] rebuild-evicted-oldest-metadata count={evicted}");
    }
    write_index_to(dir, &entries).await?;
    Ok(entries)
}

pub async fn upsert_entry(meta: AgentSessionMeta) -> Result<(), String> {
    #[cfg(test)]
    {
        let mut failed_id = FAIL_NEXT_UPSERT_SESSION.lock().await;
        if failed_id.as_deref() == Some(meta.id.as_str()) {
            *failed_id = None;
            return Err("injected index upsert failure".to_string());
        }
    }
    let _guard = INDEX_LOCK.lock().await;
    let source_revision = SESSION_SOURCE_REVISION.load(Ordering::Acquire);
    let mut entries = match read_index_from(&index_path()).await {
        Ok(entries) => entries,
        Err(_) => rebuild_index_from(index_dir().as_path()).await?,
    };
    if let Some(pos) = entries.iter().position(|e| e.id == meta.id) {
        entries[pos] = meta;
    } else {
        entries.push(meta);
    }
    write_index(&entries).await?;
    refresh_reconcile_state(&index_path(), source_revision).await;
    Ok(())
}

pub(super) async fn repair_after_upsert_failure() -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    rebuild_global_index_locked().await.map(|_| ())
}

pub(super) async fn invalidate_reconcile_fingerprint() {
    let _guard = INDEX_LOCK.lock().await;
    *INDEX_RECONCILE_FINGERPRINT.lock().await = None;
    INDEX_SOURCE_REVISION.store(u64::MAX, Ordering::Release);
}

pub(super) fn mark_document_changed() {
    SESSION_SOURCE_REVISION.fetch_add(1, Ordering::AcqRel);
}

#[cfg(test)]
pub(super) async fn fail_next_upsert_for_session(id: &str) {
    *FAIL_NEXT_UPSERT_SESSION.lock().await = Some(id.to_string());
}

pub async fn remove_entry(id: &str) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let source_revision = SESSION_SOURCE_REVISION.load(Ordering::Acquire);
    let mut entries = match read_index_from(&index_path()).await {
        Ok(entries) => entries,
        Err(_) => rebuild_index_from(index_dir().as_path()).await?,
    };
    entries.retain(|e| e.id != id);
    write_index(&entries).await?;
    refresh_reconcile_state(&index_path(), source_revision).await;
    Ok(())
}

async fn refresh_reconcile_state(path: &Path, source_revision: u64) {
    let mut last_fingerprint = INDEX_RECONCILE_FINGERPRINT.lock().await;
    *last_fingerprint = index_fingerprint(path).await;
    INDEX_SOURCE_REVISION.store(source_revision, Ordering::Release);
}

#[path = "session_index_tests.rs"]
#[cfg(test)]
mod tests;

#[path = "session_index_test_support.rs"]
#[cfg(test)]
pub(super) mod test_support;

#[path = "session_index_reconcile_tests.rs"]
#[cfg(test)]
mod reconcile_tests;
