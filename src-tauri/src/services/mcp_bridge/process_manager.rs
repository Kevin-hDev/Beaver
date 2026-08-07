use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use std::sync::Arc;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use zeroize::Zeroizing;

use super::stdio_cmd;

const MAX_PROCESSES: usize = 8;
const TTL_SECS: u64 = 600;

pub struct ProcessHandle {
    pub stdin: Arc<tokio::sync::Mutex<ChildStdin>>,
    pub reader: Arc<tokio::sync::Mutex<BufReader<ChildStdout>>>,
    pub request_lock: Arc<tokio::sync::Mutex<()>>,
}

struct PoolEntry {
    child: Child,
    last_used: Instant,
}

static POOL: std::sync::LazyLock<Mutex<HashMap<String, PoolEntry>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static HANDLES: std::sync::LazyLock<Mutex<HashMap<String, ProcessHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_alive_handle(connector_id: &str) -> Option<ProcessHandle> {
    let mut pool = POOL.lock().ok()?;
    let entry = pool.get_mut(connector_id)?;
    entry.child.id()?;
    entry.last_used = Instant::now();
    let handles = HANDLES.lock().ok()?;
    handles.get(connector_id).map(|h| ProcessHandle {
        stdin: Arc::clone(&h.stdin),
        reader: Arc::clone(&h.reader),
        request_lock: Arc::clone(&h.request_lock),
    })
}

pub fn spawn(
    connector_id: &str,
    install_command: &str,
    env_tokens: &[(String, Zeroizing<String>)],
) -> Result<ProcessHandle, String> {
    let parsed = stdio_cmd::parse_install_command(connector_id, install_command)?;

    let program_path = which::which(&parsed.program)
        .map_err(|_| "runtime requis non trouvé dans le PATH".to_string())?;

    let (child, handle) =
        super::process_spawn::spawn_program(&program_path, &parsed.args, env_tokens)?;
    register_process(connector_id, child, handle)
}

#[cfg(test)]
pub fn spawn_test_fixture(connector_id: &str) -> Result<ProcessHandle, String> {
    if connector_id != "__beaver_mcp_fixture" {
        return Err("connecteur de test non autorisé".to_string());
    }
    let fixture_root =
        dunce::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures"))
            .map_err(|_| "fixture MCP indisponible".to_string())?;
    let fixture = dunce::canonicalize(fixture_root.join("mcp-echo-server.mjs"))
        .map_err(|_| "fixture MCP indisponible".to_string())?;
    if !fixture.starts_with(&fixture_root)
        || fixture.file_name().and_then(std::ffi::OsStr::to_str) != Some("mcp-echo-server.mjs")
    {
        return Err("fixture MCP invalide".to_string());
    }
    let program = which::which("node").map_err(|_| "runtime de test indisponible".to_string())?;
    let args = vec![fixture.to_string_lossy().into_owned()];
    let (child, handle) = super::process_spawn::spawn_program(&program, &args, &[])?;
    register_process(connector_id, child, handle)
}

fn register_process(
    connector_id: &str,
    child: Child,
    handle: ProcessHandle,
) -> Result<ProcessHandle, String> {
    let evicted = {
        let mut pool = POOL.lock().map_err(|_| "erreur interne")?;
        let evicted = evict_expired_inner(&mut pool);
        let mut lru_evicted: Option<String> = None;
        if pool.len() >= MAX_PROCESSES && !pool.contains_key(connector_id) {
            if let Some(oldest_key) = pool
                .iter()
                .min_by_key(|(_, v)| v.last_used)
                .map(|(k, _)| k.clone())
            {
                if let Some(mut old) = pool.remove(&oldest_key) {
                    let _ = old.child.start_kill();
                }
                lru_evicted = Some(oldest_key);
            }
        }
        pool.insert(
            connector_id.to_string(),
            PoolEntry {
                child,
                last_used: Instant::now(),
            },
        );
        let mut all_evicted = evicted;
        if let Some(key) = lru_evicted {
            all_evicted.push(key);
        }
        all_evicted
    };

    if !evicted.is_empty() {
        if let Ok(mut handles) = HANDLES.lock() {
            for key in &evicted {
                handles.remove(key);
            }
        }
    }

    {
        let mut handles = HANDLES.lock().map_err(|_| "erreur interne")?;
        handles.insert(
            connector_id.to_string(),
            ProcessHandle {
                stdin: Arc::clone(&handle.stdin),
                reader: Arc::clone(&handle.reader),
                request_lock: Arc::clone(&handle.request_lock),
            },
        );
    }

    Ok(handle)
}

pub fn shutdown_one(connector_id: &str) {
    if let Ok(mut pool) = POOL.lock() {
        if let Some(mut entry) = pool.remove(connector_id) {
            let _ = entry.child.start_kill();
        }
    }
    if let Ok(mut handles) = HANDLES.lock() {
        handles.remove(connector_id);
    }
}

pub async fn shutdown_all() {
    let entries = POOL
        .lock()
        .map(|mut pool| pool.drain().map(|(_, entry)| entry).collect::<Vec<_>>())
        .unwrap_or_default();
    if let Ok(mut handles) = HANDLES.lock() {
        handles.clear();
    }
    futures_util::future::join_all(entries.into_iter().map(|mut entry| async move {
        crate::services::process_tree::terminate_tokio(
            &mut entry.child,
            crate::services::process_tree::ProcessKind::Mcp,
        )
        .await;
    }))
    .await;
}

fn evict_expired_inner(pool: &mut HashMap<String, PoolEntry>) -> Vec<String> {
    let expired: Vec<String> = pool
        .iter()
        .filter(|(_, e)| e.last_used.elapsed().as_secs() > TTL_SECS)
        .map(|(k, _)| k.clone())
        .collect();
    for key in &expired {
        if let Some(mut entry) = pool.remove(key) {
            let _ = entry.child.start_kill();
        }
    }
    expired
}
