use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{ServiceWorkAdmission, ServiceWorkAdmissionError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;
use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use zeroize::Zeroizing;

use super::stdio_cmd;
use super::work_supervision::{McpWorkServices, MAX_MCP_OPERATIONS, MAX_MCP_PROCESSES};

pub(super) const TTL_SECS: u64 = 600;

pub struct ProcessHandle {
    pub stdin: Arc<tokio::sync::Mutex<Option<ChildStdin>>>,
    pub reader: Arc<tokio::sync::Mutex<BufReader<ChildStdout>>>,
    pub request_lock: Arc<tokio::sync::Mutex<()>>,
    pub initialized: Arc<tokio::sync::OnceCell<()>>,
}

impl Clone for ProcessHandle {
    fn clone(&self) -> Self {
        Self {
            stdin: Arc::clone(&self.stdin),
            reader: Arc::clone(&self.reader),
            request_lock: Arc::clone(&self.request_lock),
            initialized: Arc::clone(&self.initialized),
        }
    }
}

impl ProcessHandle {
    pub(super) async fn close_stdin(&self, deadline: Instant) -> bool {
        let Ok(mut stdin) =
            tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), self.stdin.lock())
                .await
        else {
            return false;
        };
        stdin.take();
        true
    }
}

pub(super) struct PoolEntry {
    pub(super) child: Child,
    pub(super) handle: ProcessHandle,
    pub(super) last_used: Instant,
    pub(super) _admission: ServiceWorkAdmission<MAX_MCP_PROCESSES>,
}

pub(super) struct McpProcessService {
    pub(super) pool: Mutex<HashMap<String, PoolEntry>>,
    pub(super) spawn_owner: tokio::sync::Mutex<()>,
    pub(super) work: McpWorkServices,
}

impl McpProcessService {
    fn new(app_work: AppWorkSupervisor) -> Self {
        Self {
            pool: Mutex::new(HashMap::new()),
            spawn_owner: tokio::sync::Mutex::new(()),
            work: McpWorkServices::new(app_work),
        }
    }

    pub(super) fn lock_pool(&self) -> MutexGuard<'_, HashMap<String, PoolEntry>> {
        self.pool
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

static SERVICE: OnceLock<Arc<McpProcessService>> = OnceLock::new();

pub fn init(app_work: AppWorkSupervisor) -> Result<(), String> {
    SERVICE
        .set(Arc::new(McpProcessService::new(app_work)))
        .map_err(|_| "supervision MCP déjà initialisée".to_string())
}

fn global() -> Result<&'static Arc<McpProcessService>, String> {
    if let Some(service) = SERVICE.get() {
        return Ok(service);
    }
    #[cfg(test)]
    {
        static TEST_SERVICE: std::sync::LazyLock<Arc<McpProcessService>> =
            std::sync::LazyLock::new(|| {
                let coordinator = crate::app_exit::AppExitCoordinator::initialize()
                    .expect("test exit coordinator");
                Arc::new(McpProcessService::new(coordinator.work_supervisor()))
            });
        Ok(&TEST_SERVICE)
    }
    #[cfg(not(test))]
    Err("connecteur MCP indisponible".to_string())
}

pub(super) fn try_admit_operation(
) -> Result<ServiceWorkAdmission<MAX_MCP_OPERATIONS>, ServiceWorkAdmissionError> {
    global()
        .map_err(|_| ServiceWorkAdmissionError::Closing)?
        .work
        .try_admit()
}

pub async fn ensure_process(
    connector_id: &str,
    install_command: &str,
    env_tokens: &[(String, Zeroizing<String>)],
    replace_existing: bool,
) -> Result<ProcessHandle, String> {
    let parsed = stdio_cmd::parse_install_command(connector_id, install_command)?;
    let program = which::which(&parsed.program)
        .map_err(|_| "runtime requis non trouvé dans le PATH".to_string())?;
    global()?
        .ensure_spawned(
            connector_id,
            &program,
            &parsed.args,
            env_tokens,
            replace_existing,
        )
        .await
}

#[cfg(test)]
pub async fn ensure_test_fixture(connector_id: &str) -> Result<ProcessHandle, String> {
    if connector_id != "__beaver_mcp_fixture" {
        return Err("connecteur de test non autorisé".to_string());
    }
    let root =
        dunce::canonicalize(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures"))
            .map_err(|_| "fixture MCP indisponible".to_string())?;
    let fixture = dunce::canonicalize(root.join("mcp-echo-server.mjs"))
        .map_err(|_| "fixture MCP indisponible".to_string())?;
    if !fixture.starts_with(&root)
        || fixture.file_name().and_then(std::ffi::OsStr::to_str) != Some("mcp-echo-server.mjs")
    {
        return Err("fixture MCP invalide".to_string());
    }
    let program = which::which("node").map_err(|_| "runtime de test indisponible".to_string())?;
    global()?
        .ensure_spawned(
            connector_id,
            &program,
            &[fixture.to_string_lossy().into_owned()],
            &[],
            false,
        )
        .await
}

pub async fn shutdown_one(connector_id: &str) {
    if let Ok(service) = global() {
        service.shutdown_one(connector_id).await;
    }
}

pub async fn stop_and_wait(deadline: Instant) -> bool {
    let Some(service) = SERVICE.get() else {
        return true;
    };
    service.stop_and_wait(deadline).await
}

#[cfg(test)]
pub fn process_id_for_test(connector_id: &str) -> Option<u32> {
    global().ok()?.process_id(connector_id)
}
