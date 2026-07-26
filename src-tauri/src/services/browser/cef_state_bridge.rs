use super::{
    browser_events::{BrowserSessionEvent, EVENT_VERSION, TAB_STATE_CHANGED_EVENT},
    browser_view_key::BrowserViewKey,
    runtime_revision::RuntimeStamp,
    session_service::BrowserSessionService,
    session_types::BrowserRuntimeTabUpdate,
};
use tauri::{Emitter, Manager};

pub(super) fn submit_runtime_update(
    app: &tauri::AppHandle,
    key: BrowserViewKey,
    stamp: RuntimeStamp,
    update: BrowserRuntimeTabUpdate,
) {
    let service = app.state::<BrowserSessionService>().inner().clone();
    let event_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let session_id = key.session_id.clone();
        let tab_id = key.tab_id.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            service.update_runtime(&session_id, &tab_id, stamp, update)
        })
        .await;
        if let Ok(Ok(Some(session))) = result {
            let _ = event_app.emit(
                TAB_STATE_CHANGED_EVENT,
                BrowserSessionEvent {
                    event_version: EVENT_VERSION,
                    conversation_id: key.session_id,
                    session,
                },
            );
        }
    });
}

pub(super) fn mark_view_released(app: &tauri::AppHandle, key: BrowserViewKey, stamp: RuntimeStamp) {
    let service = app.state::<BrowserSessionService>().inner().clone();
    let event_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let session_id = key.session_id.clone();
        let tab_id = key.tab_id.clone();
        let result = tauri::async_runtime::spawn_blocking(move || {
            service.mark_released(&session_id, &tab_id, stamp)
        })
        .await;
        if let Ok(Ok(Some(session))) = result {
            let _ = event_app.emit(
                TAB_STATE_CHANGED_EVENT,
                BrowserSessionEvent {
                    event_version: EVENT_VERSION,
                    conversation_id: key.session_id,
                    session,
                },
            );
        }
    });
}
