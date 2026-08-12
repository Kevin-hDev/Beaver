use super::process_manager::{McpProcessService, PoolEntry, ProcessHandle, TTL_SECS};
use super::work_supervision::MAX_MCP_PROCESSES;
use futures_util::future::join_all;
use std::path::Path;
use std::time::Instant;
use zeroize::Zeroizing;

const MCP_PROCESS_STOP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

impl McpProcessService {
    pub(super) async fn ensure_spawned(
        &self,
        connector_id: &str,
        program: &Path,
        args: &[String],
        env_tokens: &[(String, Zeroizing<String>)],
        replace_existing: bool,
    ) -> Result<ProcessHandle, String> {
        let _owner = self.spawn_owner.lock().await;
        if !replace_existing {
            if let Some(handle) = self.alive_handle(connector_id)? {
                return Ok(handle);
            }
        }
        if let Some(entry) = self.take_one(connector_id) {
            terminate_entry(entry, process_deadline()).await;
        }
        let evicted = self.take_evictions();
        let deadline = process_deadline();
        join_all(
            evicted
                .into_iter()
                .map(|entry| terminate_entry(entry, deadline)),
        )
        .await;

        let admission = self
            .work
            .try_admit_process()
            .map_err(|_| "connecteur MCP indisponible".to_string())?;
        let cancellation = admission.cancellation();
        if cancellation.is_cancelled() {
            return Err("connecteur MCP indisponible".to_string());
        }
        let (child, handle) =
            super::process_spawn::spawn_program(program, args, env_tokens).await?;
        let entry = PoolEntry {
            child,
            handle: handle.clone(),
            last_used: Instant::now(),
            _admission: admission,
        };
        if cancellation.is_cancelled() {
            terminate_entry(entry, process_deadline()).await;
            return Err("connecteur MCP indisponible".to_string());
        }
        let rejected = {
            let mut pool = self.lock_pool();
            if pool.len() >= MAX_MCP_PROCESSES || pool.contains_key(connector_id) {
                Some(entry)
            } else {
                pool.insert(connector_id.to_string(), entry);
                None
            }
        };
        if let Some(entry) = rejected {
            terminate_entry(entry, process_deadline()).await;
            return Err("connecteur MCP indisponible".to_string());
        }
        Ok(handle)
    }

    pub(super) async fn shutdown_one(&self, connector_id: &str) {
        let _owner = self.spawn_owner.lock().await;
        if let Some(entry) = self.take_one(connector_id) {
            terminate_entry(entry, process_deadline()).await;
        }
    }

    pub(super) async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        let entries = self.drain_pool();
        let first_stopped = terminate_all(entries, deadline).await;
        let Ok(owner) = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            self.spawn_owner.lock(),
        )
        .await
        else {
            let _ = self.work.stop_and_wait(deadline).await;
            return false;
        };
        let entries = self.drain_pool();
        drop(owner);
        let second_stopped = terminate_all(entries, deadline).await;
        first_stopped && second_stopped && self.work.stop_and_wait(deadline).await
    }

    #[cfg(test)]
    pub(super) fn process_id(&self, connector_id: &str) -> Option<u32> {
        self.lock_pool().get(connector_id)?.child.id()
    }

    fn alive_handle(&self, connector_id: &str) -> Result<Option<ProcessHandle>, String> {
        let mut pool = self.lock_pool();
        let Some(entry) = pool.get_mut(connector_id) else {
            return Ok(None);
        };
        match entry.child.try_wait() {
            Ok(None) => {
                entry.last_used = Instant::now();
                Ok(Some(entry.handle.clone()))
            }
            Ok(Some(_)) => {
                pool.remove(connector_id);
                Ok(None)
            }
            Err(_) => Err("connecteur MCP indisponible".to_string()),
        }
    }

    fn take_one(&self, connector_id: &str) -> Option<PoolEntry> {
        self.lock_pool().remove(connector_id)
    }

    fn take_evictions(&self) -> Vec<PoolEntry> {
        let mut pool = self.lock_pool();
        let mut keys = pool
            .iter()
            .filter(|(_, entry)| entry.last_used.elapsed().as_secs() > TTL_SECS)
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        if pool.len().saturating_sub(keys.len()) >= MAX_MCP_PROCESSES {
            if let Some(oldest) = pool
                .iter()
                .filter(|(key, _)| !keys.contains(key))
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone())
            {
                keys.push(oldest);
            }
        }
        keys.into_iter()
            .filter_map(|key| pool.remove(&key))
            .collect()
    }

    fn drain_pool(&self) -> Vec<PoolEntry> {
        self.lock_pool().drain().map(|(_, entry)| entry).collect()
    }
}

pub(super) async fn terminate_entry(mut entry: PoolEntry, deadline: Instant) -> bool {
    if !entry.handle.close_stdin(deadline).await {
        let _ = entry.child.start_kill();
        return false;
    }
    tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        crate::services::process_tree::terminate_tokio(
            &mut entry.child,
            crate::services::process_tree::ProcessKind::Mcp,
        ),
    )
    .await
    .is_ok()
}

async fn terminate_all(entries: Vec<PoolEntry>, deadline: Instant) -> bool {
    join_all(
        entries
            .into_iter()
            .map(|entry| terminate_entry(entry, deadline)),
    )
    .await
    .into_iter()
    .all(|stopped| stopped)
}

fn process_deadline() -> Instant {
    Instant::now() + MCP_PROCESS_STOP_TIMEOUT
}
