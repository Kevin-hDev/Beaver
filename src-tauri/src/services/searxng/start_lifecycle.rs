use super::lifecycle::{base_url, shutdown_error, SearxngHandle, SearxngSidecar};
use super::start_readiness::{ensure_start_active, run_if_start_active};
use crate::services::work_registry::ServiceWorkCancellation;
use std::sync::atomic::Ordering;
use std::time::Duration;

const ORPHAN_RECOVERY_TIMEOUT: Duration = Duration::from_secs(3);
const SIDECAR_START_TIMEOUT: Duration = Duration::from_secs(12);
const IDENTITY_STABILITY_TIMEOUT: Duration = Duration::from_millis(250);

impl SearxngSidecar {
    pub(super) async fn ensure_running(
        &self,
        app: &tauri::AppHandle,
        cancel: &ServiceWorkCancellation,
    ) -> Result<String, String> {
        let _start = tokio::select! {
            guard = self.start_gate.lock() => guard,
            _ = cancel.cancelled() => return Err(shutdown_error()),
        };
        let generation = self.publication_generation.load(Ordering::Acquire);
        if let Some(url) = self.running_url().await? {
            return Ok(url);
        }
        ensure_start_active(self, cancel, generation)?;
        if let Some(error) = super::startup_failure::recent() {
            return Err(error);
        }

        let recovery_cancel = cancel.clone();
        let recovery_deadline = std::time::Instant::now() + ORPHAN_RECOVERY_TIMEOUT;
        super::startup::run_blocking(move || {
            super::process::recover_orphan_sidecar(recovery_deadline, &recovery_cancel)
        })
        .await?;
        ensure_start_active(self, cancel, generation)?;
        let source = super::paths::source_dir(app)?;
        let python = super::runtime::ensure_runtime(&source, cancel).await?;
        ensure_start_active(self, cancel, generation)?;
        let startup_deadline = tokio::time::Instant::now() + SIDECAR_START_TIMEOUT;
        let port = super::settings::find_free_port()?;
        let settings = super::settings::write_settings(port)?;
        let admission = self.work.try_admit_server().map_err(|_| shutdown_error())?;
        let mut child = super::process::spawn(&python, &source, &settings, port).await?;
        let pid = match child.id() {
            Some(pid) => pid,
            None => {
                super::process::kill_child_process(child).await;
                return Err(super::error_codes::START_FAILED.to_string());
            }
        };
        let identity_deadline =
            startup_deadline.min(tokio::time::Instant::now() + IDENTITY_STABILITY_TIMEOUT);
        match super::start_process_receipt::stabilize(pid, identity_deadline, cancel).await {
            Ok(_) => {}
            Err(error) => {
                super::process::kill_child_process(child).await;
                return Err(error);
            }
        }
        let url = base_url(port);
        if let Err(error) =
            super::startup::wait_until_ready(&url, &mut child, cancel, startup_deadline).await
        {
            super::startup_failure::remember(&error);
            super::process::kill_child_process(child).await;
            return Err(error);
        }
        if run_if_start_active(self, cancel, generation, || {
            if let Err(error) = super::runtime_environment::RuntimeEnvironment::mark_started() {
                ::log::warn!(
                    "[searxng] runtime previous cleanup category={}",
                    error.category()
                );
            }
        })
        .is_err()
        {
            super::process::kill_child_process(child).await;
            return Err(shutdown_error());
        }
        let handle = SearxngHandle {
            child,
            port,
            _admission: admission,
        };
        if let Err(handle) = self.publish(handle, generation, cancel).await {
            let handle = *handle;
            super::process::kill_child_process(handle.child).await;
            return Err(shutdown_error());
        }
        ::log::info!("[searxng] sidecar démarré pid={pid} port={port}");
        super::startup_failure::clear();
        Ok(url)
    }

    async fn running_url(&self) -> Result<Option<String>, String> {
        let mut guard = self.process.lock().await;
        let Some(handle) = guard.as_mut() else {
            return Ok(None);
        };
        match handle.child.try_wait() {
            Ok(None) => Ok(Some(base_url(handle.port))),
            Ok(Some(_)) => {
                *guard = None;
                Ok(None)
            }
            Err(_) => Err(super::error_codes::PROCESS_STATE_UNAVAILABLE.to_string()),
        }
    }

    async fn publish(
        &self,
        handle: SearxngHandle,
        generation: u64,
        cancel: &ServiceWorkCancellation,
    ) -> Result<(), Box<SearxngHandle>> {
        let mut guard = self.process.lock().await;
        if ensure_start_active(self, cancel, generation).is_err() || guard.is_some() {
            // Rejection returns ownership to cleanup; boxing keeps every result
            // crossing this uncommon async boundary small.
            return Err(Box::new(handle));
        }
        *guard = Some(handle);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) async fn start_test_process_for_test(&self) -> Result<u32, String> {
        let admission = self
            .work
            .try_admit_server()
            .map_err(|_| "fixture SearXNG indisponible".to_string())?;
        let child = super::process::spawn_test_fixture().await?;
        let pid = child
            .id()
            .ok_or_else(|| "fixture SearXNG indisponible".to_string())?;
        *self.process.lock().await = Some(SearxngHandle {
            child,
            port: 0,
            _admission: admission,
        });
        Ok(pid)
    }

    #[cfg(test)]
    pub(crate) async fn published_pid_for_test(&self) -> Option<u32> {
        self.process.lock().await.as_ref()?.child.id()
    }

    #[cfg(test)]
    pub(crate) async fn reject_stale_test_publication_for_test(&self) -> Result<u32, String> {
        let admission = self
            .work
            .try_admit_server()
            .map_err(|_| "fixture SearXNG indisponible".to_string())?;
        let cancel = admission.cancellation();
        let child = super::process::spawn_test_fixture().await?;
        let pid = child
            .id()
            .ok_or_else(|| "fixture SearXNG indisponible".to_string())?;
        let generation = self.publication_generation.load(Ordering::Acquire);
        self.publication_generation.fetch_add(1, Ordering::AcqRel);
        let handle = SearxngHandle {
            child,
            port: 0,
            _admission: admission,
        };
        let rejected = *self
            .publish(handle, generation, &cancel)
            .await
            .expect_err("stale generation must lose publication");
        super::process::kill_child_process(rejected.child).await;
        Ok(pid)
    }

    #[cfg(test)]
    pub(crate) async fn suspend_test_start_before_publication_for_test(
        &self,
        started: tokio::sync::oneshot::Sender<u32>,
        release: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<(), String> {
        let run_state = self.clone();
        self.work
            .run_start(move |_cancel| async move {
                let _start = run_state.start_gate.lock().await;
                let child = super::process::spawn_test_fixture().await?;
                let pid = child
                    .id()
                    .ok_or_else(|| "fixture SearXNG indisponible".to_string())?;
                let _ = started.send(pid);
                let _ = release.await;
                drop(child);
                Ok(())
            })
            .await
            .map_err(|_| "fixture SearXNG interrompue".to_string())?
    }
}
