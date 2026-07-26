use crate::services::browser::{
    BrowserCapability, BrowserCommandError, BrowserNavigationAction, BrowserSessionService,
    BrowserSessionState, BrowserSurfaceRequest, BrowserTabCreation, LocalSiteScanResult,
    LocalSiteScanner, LOCAL_SITES_CHANGED_EVENT,
};
use tauri::Emitter;

#[tauri::command]
pub fn browser_capability(app: tauri::AppHandle) -> BrowserCapability {
    crate::services::browser::capability(&app)
}

#[tauri::command]
pub async fn browser_surface(
    app: tauri::AppHandle,
    request: BrowserSurfaceRequest,
) -> Result<(), BrowserCommandError> {
    crate::services::browser::apply_surface(app, request).await
}

#[tauri::command]
pub async fn browser_open_session(
    service: tauri::State<'_, BrowserSessionService>,
    conversation_id: String,
) -> Result<BrowserSessionState, BrowserCommandError> {
    run_blocking(service.inner().clone(), move |service| {
        service.open(&conversation_id)
    })
    .await
}

#[tauri::command]
pub async fn browser_create_tab(
    service: tauri::State<'_, BrowserSessionService>,
    conversation_id: String,
    replace_tab_id: Option<String>,
) -> Result<BrowserTabCreation, BrowserCommandError> {
    run_blocking(service.inner().clone(), move |service| {
        service.create_tab(&conversation_id, replace_tab_id.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn browser_activate_tab(
    service: tauri::State<'_, BrowserSessionService>,
    conversation_id: String,
    tab_id: String,
) -> Result<BrowserSessionState, BrowserCommandError> {
    run_blocking(service.inner().clone(), move |service| {
        service.activate_tab(&conversation_id, &tab_id)
    })
    .await
}

#[tauri::command]
pub async fn browser_close_tab(
    app: tauri::AppHandle,
    service: tauri::State<'_, BrowserSessionService>,
    conversation_id: String,
    tab_id: String,
) -> Result<BrowserSessionState, BrowserCommandError> {
    crate::services::browser::close_native_view(app, conversation_id.clone(), tab_id.clone())
        .await?;
    run_blocking(service.inner().clone(), move |service| {
        service.close_tab(&conversation_id, &tab_id)
    })
    .await
}

#[tauri::command]
pub async fn browser_navigation_action(
    app: tauri::AppHandle,
    conversation_id: String,
    tab_id: String,
    action: BrowserNavigationAction,
) -> Result<(), BrowserCommandError> {
    crate::services::browser::run_navigation_action(app, conversation_id, tab_id, action).await
}

#[tauri::command]
pub async fn browser_navigate(
    app: tauri::AppHandle,
    service: tauri::State<'_, BrowserSessionService>,
    conversation_id: String,
    tab_id: String,
    url: String,
) -> Result<BrowserSessionState, BrowserCommandError> {
    let native_conversation_id = conversation_id.clone();
    let native_tab_id = tab_id.clone();
    let native_url = url.clone();
    let session = run_blocking(service.inner().clone(), move |service| {
        service.navigate(&conversation_id, &tab_id, &url)
    })
    .await?;
    crate::services::browser::navigate_native_view(
        app,
        native_conversation_id,
        native_tab_id,
        native_url,
    )
    .await?;
    Ok(session)
}

async fn run_blocking<T: Send + 'static>(
    service: BrowserSessionService,
    operation: impl FnOnce(BrowserSessionService) -> Result<T, BrowserCommandError> + Send + 'static,
) -> Result<T, BrowserCommandError> {
    tauri::async_runtime::spawn_blocking(move || operation(service))
        .await
        .map_err(|_| BrowserCommandError::Internal)?
}

#[tauri::command]
pub async fn browser_detect_local_sites(
    app: tauri::AppHandle,
    scanner: tauri::State<'_, LocalSiteScanner>,
    home_visible: bool,
) -> Result<LocalSiteScanResult, BrowserCommandError> {
    if !matches!(
        crate::services::browser::capability(&app),
        BrowserCapability::Ready { .. }
    ) {
        return Err(BrowserCommandError::Unavailable);
    }
    let result = scanner
        .inner()
        .clone()
        .scan(home_visible)
        .await
        .map_err(|_| BrowserCommandError::Internal)?;
    if result.changed {
        let _ = app.emit(LOCAL_SITES_CHANGED_EVENT, result.clone());
    }
    Ok(result)
}
