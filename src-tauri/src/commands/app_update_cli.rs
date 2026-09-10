use super::app_update::AppUpdateInfo;

pub(crate) async fn download_and_launch(update: AppUpdateInfo) -> Result<(), String> {
    let cancellation = crate::services::work_registry::ServiceWorkCancellation::standalone();
    let temporary = super::app_update_install::download_verified_update(
        &update.asset_url,
        &cancellation,
        |_| {},
    )
    .await?;
    super::app_update_helper_process::launch_update_helper_for_cli(temporary.path()).await?;
    let _ = temporary.persist();
    Ok(())
}
