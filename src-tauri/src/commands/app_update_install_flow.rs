use super::app_update_install::{download_verified_asset, DownloadProgress};
use super::app_update_progress::{finish as finish_progress, running_operation};
use super::app_update_source::AssetReference;
use crate::services::update_progress::{
    UpdateOperationPhase, UpdateOperationStatus, UpdateProgressMode, UpdateProgressRuntime,
};
use tauri::ipc::Channel;

pub(super) async fn run(
    app: tauri::AppHandle,
    asset: AssetReference,
    on_progress: Channel<DownloadProgress>,
    updates: crate::services::update_handoff::AppUpdateRuntime,
    progress: UpdateProgressRuntime,
    identity: (String, String),
    cancellation: crate::services::work_registry::ServiceWorkCancellation,
) -> Result<(), String> {
    let (id, label) = identity;
    let tmp = download_verified_asset(asset, &cancellation, |download| {
        let _ = on_progress.send(download.clone());
        let percent = u128::from(download.completed)
            .saturating_mul(100)
            .checked_div(u128::from(download.total))
            .unwrap_or(0)
            .min(100) as u8;
        let _ = progress.upsert(
            &app,
            running_operation(
                &id,
                &label,
                UpdateOperationPhase::Downloading,
                UpdateProgressMode::Determinate,
                Some(percent),
                true,
            ),
        );
    })
    .await?;
    progress.upsert(
        &app,
        running_operation(
            &id,
            &label,
            UpdateOperationPhase::Restarting,
            UpdateProgressMode::Indeterminate,
            None,
            false,
        ),
    )?;
    let helper =
        super::app_update_helper_process::spawn_update_helper(&app, tmp.path(), &cancellation)
            .await?;
    helper.commit(updates.handoff(), &cancellation)?;
    let _ = tmp.persist();
    if finish_progress(&app, &progress, &id, UpdateOperationStatus::Completed, None).is_err() {
        log::warn!("update_progress_finalization_failed");
    }
    crate::app_exit::request(&app, 0);
    Ok(())
}
