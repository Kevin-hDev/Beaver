use super::lifecycle::{base_url, shutdown_error, SearxngHandle, SearxngSidecar};
use super::start_readiness::{ensure_start_active, run_if_start_active};
use crate::services::work_registry::ServiceWorkCancellation;
use std::sync::atomic::Ordering;

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

        super::startup::run_blocking(super::process::kill_orphan_sidecar).await?;
        ensure_start_active(self, cancel, generation)?;
        let source = super::paths::source_dir(app)?;
        let python = super::runtime::ensure_runtime(&source, cancel).await?;
        ensure_start_active(self, cancel, generation)?;
        let port = super::settings::find_free_port()?;
        let settings = super::settings::write_settings(port)?;
        let admission = self.work.try_admit_server().map_err(|_| shutdown_error())?;
        let mut child = super::process::spawn(&python, &source, &settings, port).await?;
        let pid = child
            .id()
            .ok_or_else(|| "SearXNG: démarrage impossible".to_string())?;
        let url = base_url(port);
        if let Err(error) = super::startup::wait_until_ready(&url, &mut child, cancel).await {
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
        if let Err(handle) = self
            .publish_with_pid_save(handle, pid, generation, cancel, super::process::save_pid)
            .await
        {
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
            Err(_) => Err("SearXNG: état processus illisible".to_string()),
        }
    }

    async fn publish_with_pid_save<Persist>(
        &self,
        handle: SearxngHandle,
        pid: u32,
        generation: u64,
        cancel: &ServiceWorkCancellation,
        persist: Persist,
    ) -> Result<(), SearxngHandle>
    where
        Persist: FnOnce(u32) + Send + 'static,
    {
        let mut guard = self.process.lock().await;
        if ensure_start_active(self, cancel, generation).is_err() || guard.is_some() {
            return Err(handle);
        }
        *guard = Some(handle);
        drop(guard);

        // Persistence is deliberately outside the process lock so status and
        // shutdown remain available while the filesystem is slow.
        let _ = super::startup::run_blocking(move || persist(pid)).await;
        if self.publication_generation.load(Ordering::Acquire) != generation {
            // start_gate still excludes a newer start, so this cannot erase its PID.
            super::process::clear_pid_file();
        }
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
    pub(crate) async fn publish_test_process_with_pid_save_for_test<Persist>(
        &self,
        persist: Persist,
    ) -> Result<(), String>
    where
        Persist: FnOnce(u32) + Send + 'static,
    {
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
        let handle = SearxngHandle {
            child,
            port: 0,
            _admission: admission,
        };
        self.publish_with_pid_save(handle, pid, generation, &cancel, persist)
            .await
            .map_err(|_| "fixture SearXNG interrompue".to_string())
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
        let rejected = self
            .publish_with_pid_save(handle, pid, generation, &cancel, |_| {})
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
