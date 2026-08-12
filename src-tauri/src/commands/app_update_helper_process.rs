use super::app_update_helper::{
    copy_helper_while, current_install_directory, helper_resource_name, TemporaryHelper,
};
use crate::services::process_identity::ProcessIdentity;
use crate::services::update_handoff::UpdateHandoff;
use crate::services::work_registry::ServiceWorkCancellation;
use std::path::Path;
use std::process::{Child, Stdio};
use tauri::Manager;

pub(crate) fn spawn_update_helper(
    app: &tauri::AppHandle,
    asset: &Path,
    cancellation: &ServiceWorkCancellation,
) -> Result<SpawnedUpdateHelper, String> {
    check_cancelled(cancellation)?;
    let resource_root = app.path().resource_dir().map_err(|_| install_error())?;
    let source = resource_root
        .join("target/updater-helper")
        .join(helper_resource_name());
    let helper = copy_helper_while(&source, &resource_root, &std::env::temp_dir(), || {
        cancellation.is_cancelled()
    })
    .map_err(|error| {
        if cancellation.is_cancelled() {
            download_error()
        } else {
            error
        }
    })?;
    check_cancelled(cancellation)?;
    let working_directory = current_install_directory()?;
    let mut command = crate::services::background_command::new(helper.path());
    command
        .arg("--apply-update")
        .arg(asset)
        .arg("--parent-pid")
        .arg(std::process::id().to_string())
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    crate::services::process_tree::configure(&mut command);
    check_cancelled(cancellation)?;
    let mut child = command.spawn().map_err(|_| install_error())?;
    let identity = ProcessIdentity::capture_child(child.id(), std::process::id(), helper.path());
    if cancellation.is_cancelled() || identity.is_none() {
        crate::services::process_tree::terminate(
            &mut child,
            crate::services::process_tree::ProcessKind::UpdateHelper,
        );
        return Err(if cancellation.is_cancelled() {
            download_error()
        } else {
            install_error()
        });
    }
    Ok(SpawnedUpdateHelper {
        child: Some(child),
        helper: Some(helper),
        identity: identity.expect("identity checked above"),
    })
}

pub(crate) struct SpawnedUpdateHelper {
    child: Option<Child>,
    helper: Option<TemporaryHelper>,
    identity: ProcessIdentity,
}

impl SpawnedUpdateHelper {
    pub(crate) fn commit(
        mut self,
        handoff: &UpdateHandoff,
        cancellation: &ServiceWorkCancellation,
    ) -> Result<(), String> {
        let child = self.child.take().ok_or_else(install_error)?;
        if let Err(child) = handoff.transfer(self.identity.clone(), child, cancellation) {
            self.child = Some(child);
            return Err(if cancellation.is_cancelled() {
                download_error()
            } else {
                install_error()
            });
        }
        if let Some(helper) = self.helper.take() {
            helper.persist();
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn from_test_child(child: Child, identity: ProcessIdentity) -> Self {
        Self {
            child: Some(child),
            helper: Some(super::app_update_helper::test_temporary_helper()),
            identity,
        }
    }
}

impl Drop for SpawnedUpdateHelper {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            crate::services::process_tree::terminate(
                &mut child,
                crate::services::process_tree::ProcessKind::UpdateHelper,
            );
        }
    }
}

fn check_cancelled(cancellation: &ServiceWorkCancellation) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err(download_error())
    } else {
        Ok(())
    }
}

fn download_error() -> String {
    "update-download-error".to_string()
}

fn install_error() -> String {
    "update-install-error".to_string()
}

#[cfg(test)]
#[path = "app_update_helper_process_tests.rs"]
mod tests;
