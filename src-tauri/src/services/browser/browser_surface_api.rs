use super::browser_api_types::{
    BrowserCommandError, BrowserNavigationAction, BrowserSurfaceRequest,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::{browser_view_key::BrowserViewKey, runtime_handle::BrowserCapability};

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub async fn apply_surface(
    app: tauri::AppHandle,
    request: BrowserSurfaceRequest,
) -> Result<(), BrowserCommandError> {
    ensure_ready(&app)?;
    request.bounds.validate().map_err(invalid)?;
    let key = BrowserViewKey::new(request.conversation_id, request.tab_id).map_err(invalid)?;
    let url = request
        .url
        .as_deref()
        .map(super::url_policy::validate_browser_url)
        .transpose()
        .map_err(invalid)?;
    run_on_main(app, move |main_app| {
        super::cef_engine::apply_surface(main_app, key, url, request.bounds)
    })
    .await
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub async fn apply_surface(
    _app: tauri::AppHandle,
    request: BrowserSurfaceRequest,
) -> Result<(), BrowserCommandError> {
    let BrowserSurfaceRequest {
        conversation_id,
        tab_id,
        url,
        bounds,
    } = request;
    drop((conversation_id, tab_id, url, bounds));
    Err(BrowserCommandError::Unavailable)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub async fn run_navigation_action(
    app: tauri::AppHandle,
    conversation_id: String,
    tab_id: String,
    action: BrowserNavigationAction,
) -> Result<(), BrowserCommandError> {
    ensure_ready(&app)?;
    let key = BrowserViewKey::new(conversation_id, tab_id).map_err(invalid)?;
    run_on_main(app, move |_| {
        super::cef_engine::navigation_action(&key, action)
    })
    .await
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub async fn navigate_native_view(
    app: tauri::AppHandle,
    conversation_id: String,
    tab_id: String,
    raw_url: String,
) -> Result<(), BrowserCommandError> {
    ensure_ready(&app)?;
    let key = BrowserViewKey::new(conversation_id, tab_id).map_err(invalid)?;
    let url = super::url_policy::validate_browser_url(&raw_url).map_err(invalid)?;
    run_on_main(app, move |_| super::cef_engine::navigate(&key, &url)).await
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub async fn navigate_native_view(
    _app: tauri::AppHandle,
    _conversation_id: String,
    _tab_id: String,
    _raw_url: String,
) -> Result<(), BrowserCommandError> {
    Err(BrowserCommandError::Unavailable)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub async fn run_navigation_action(
    _app: tauri::AppHandle,
    _conversation_id: String,
    _tab_id: String,
    _action: BrowserNavigationAction,
) -> Result<(), BrowserCommandError> {
    Err(BrowserCommandError::Unavailable)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub async fn close_native_view(
    app: tauri::AppHandle,
    conversation_id: String,
    tab_id: String,
) -> Result<(), BrowserCommandError> {
    let key = BrowserViewKey::new(conversation_id, tab_id).map_err(invalid)?;
    run_on_main(app, move |main_app| {
        super::cef_engine::close_view(main_app, &key)
    })
    .await
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub async fn close_native_view(
    _app: tauri::AppHandle,
    _conversation_id: String,
    _tab_id: String,
) -> Result<(), BrowserCommandError> {
    Err(BrowserCommandError::Unavailable)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
async fn run_on_main(
    app: tauri::AppHandle,
    operation: impl FnOnce(&tauri::AppHandle) -> Result<(), ()> + Send + 'static,
) -> Result<(), BrowserCommandError> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let main_app = app.clone();
    app.run_on_main_thread(move || {
        let _ = sender.send(operation(&main_app));
    })
    .map_err(|_| BrowserCommandError::Unavailable)?;
    receiver
        .await
        .map_err(|_| BrowserCommandError::Internal)?
        .map_err(|_| BrowserCommandError::Unavailable)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn ensure_ready(app: &tauri::AppHandle) -> Result<(), BrowserCommandError> {
    matches!(super::capability(app), BrowserCapability::Ready { .. })
        .then_some(())
        .ok_or(BrowserCommandError::Unavailable)
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn invalid(_: ()) -> BrowserCommandError {
    BrowserCommandError::InvalidInput
}
