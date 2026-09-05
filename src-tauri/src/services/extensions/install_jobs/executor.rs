use super::super::{npm_runner::NpmRunner, ui_builder::UiBuildRuntime};
use super::{
    checkpoint::InstallCheckpoint, InstallControl, InstallExecutor, InstallFuture,
    InstallInterruption, InstallOutcome, InstallRequest,
};
use std::sync::Arc;

pub(super) struct ProductionExecutor {
    npm: NpmRunner,
    ui: UiBuildRuntime,
}
impl ProductionExecutor {
    #[cfg(test)]
    pub(super) fn for_test(npm: NpmRunner, ui: UiBuildRuntime) -> Arc<dyn InstallExecutor> {
        Arc::new(Self { npm, ui })
    }
    pub(super) fn resolve(app: &tauri::AppHandle) -> Result<Arc<dyn InstallExecutor>, String> {
        Ok(Arc::new(Self {
            npm: NpmRunner::resolve(app).map_err(|error| error.code())?,
            ui: UiBuildRuntime::resolve(app).map_err(|error| error.code())?,
        }))
    }
}
impl InstallExecutor for ProductionExecutor {
    fn execute(&self, request: InstallRequest, control: InstallControl) -> InstallFuture {
        let npm = self.npm.clone();
        let ui = self.ui.clone();
        Box::pin(async move {
            let work = control.store.work.clone();
            match super::owned_work::spawn(&work, move || execute(request, control, npm, ui)) {
                Ok(receiver) => receiver.await.unwrap_or(InstallOutcome {
                    result: Err(InstallInterruption::Failed),
                    cleanup_confirmed: false,
                }),
                Err(_) => InstallOutcome {
                    result: Err(InstallInterruption::AppClosing),
                    cleanup_confirmed: true,
                },
            }
        })
    }
}

fn execute(
    request: InstallRequest,
    control: InstallControl,
    npm: NpmRunner,
    ui: UiBuildRuntime,
) -> InstallOutcome {
    let mut checkpoint = match control.saved() {
        Ok(Some(checkpoint)) => checkpoint,
        Ok(None) => InstallCheckpoint {
            version: super::checkpoint::FORMAT,
            token: uuid::Uuid::new_v4().simple().to_string(),
            budget_bytes: super::super::managed_tree::MAX_TOTAL_BYTES,
            ..Default::default()
        },
        Err(error) => {
            return InstallOutcome {
                result: Err(error),
                cleanup_confirmed: false,
            }
        }
    };
    let result = super::materialize::prepare(&request, &control, &mut checkpoint, &npm, &ui)
        .and_then(|record| publish(&control, &checkpoint, record));
    // Always re-read ownership changed by process callbacks. A stale local copy
    // cannot attest that a producer has stopped or its identity was saved.
    let mut checkpoint = control.saved().ok().flatten().unwrap_or(checkpoint);
    let durable = control.store.lock().is_ok_and(|state| !state.durable_error);
    if !durable || checkpoint.native_process.is_some() {
        return InstallOutcome {
            result: Err(InstallInterruption::Failed),
            cleanup_confirmed: false,
        };
    }
    checkpoint.producer_active = false;
    let cleanup_confirmed = super::cleanup::run(&checkpoint).is_ok();
    checkpoint.cleanup_unconfirmed = !cleanup_confirmed;
    let persisted = control.save(checkpoint).is_ok();
    let result = if result.is_err() && control.app_cancel.is_cancelled() {
        Err(InstallInterruption::AppClosing)
    } else if result.is_err() && control.is_cancelled() {
        Err(InstallInterruption::Cancelled)
    } else {
        result
    };
    InstallOutcome {
        result,
        cleanup_confirmed: cleanup_confirmed && persisted,
    }
}

fn publish(
    control: &InstallControl,
    checkpoint: &InstallCheckpoint,
    record: super::super::types::ExtensionRecord,
) -> Result<String, InstallInterruption> {
    if let Some(previous) = &checkpoint.previous {
        let runtime = super::super::runtime::global().map_err(|_| InstallInterruption::Failed)?;
        let identity =
            super::super::host_identity::HostIdentity::ThirdParty(previous.manifest.id.clone());
        let stopped = tokio::runtime::Handle::current().block_on(runtime.revoke_extension(
            &identity,
            std::time::Instant::now() + std::time::Duration::from_secs(2),
        ));
        if !stopped {
            return Err(InstallInterruption::Failed);
        }
    }
    control.publish(|| {
        let id = record.manifest.id.clone();
        if let Some(previous) = &checkpoint.previous {
            let replacement = super::super::installer_record::for_update(previous, record);
            super::super::registry::replace_user(previous, replacement)
                .map_err(|_| InstallInterruption::Failed)?;
        } else {
            super::super::registry_managed::add(record).map_err(|_| InstallInterruption::Failed)?;
        }
        Ok(id)
    })
}
