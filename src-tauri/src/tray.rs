use crate::services::{brand::DISPLAY_NAME, gateway::GatewayService};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

const SHOW_MENU_ID: &str = "show";
const GATEWAY_MENU_ID: &str = "gateway-toggle";
const QUIT_MENU_ID: &str = "quit";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrayMenuAction {
    Show,
    ToggleGateway,
    Quit,
    Ignore,
}

fn menu_action(id: &str) -> TrayMenuAction {
    match id {
        SHOW_MENU_ID => TrayMenuAction::Show,
        GATEWAY_MENU_ID => TrayMenuAction::ToggleGateway,
        QUIT_MENU_ID => TrayMenuAction::Quit,
        _ => TrayMenuAction::Ignore,
    }
}

fn tray_lang() -> &'static str {
    let locale = sys_locale::get_locale().unwrap_or_default();
    if locale.to_lowercase().starts_with("fr") {
        "fr"
    } else {
        "en"
    }
}

struct TrayLabels {
    show: &'static str,
    gateway: &'static str,
    quit: &'static str,
}

fn labels() -> TrayLabels {
    if tray_lang() == "fr" {
        TrayLabels {
            show: "Afficher",
            gateway: "Gateway",
            quit: "Quitter",
        }
    } else {
        TrayLabels {
            show: "Show",
            gateway: "Gateway",
            quit: "Quit",
        }
    }
}

fn restore_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn should_restore_from_tray_click(button: &MouseButton, button_state: &MouseButtonState) -> bool {
    // Le clic droit appartient au menu natif ; seule l'action gauche terminée restaure Beaver.
    matches!(
        (button, button_state),
        (MouseButton::Left, MouseButtonState::Up)
    )
}

pub fn create_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let l = labels();
    let show = MenuItem::with_id(app, SHOW_MENU_ID, l.show, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let gw = MenuItem::with_id(app, GATEWAY_MENU_ID, l.gateway, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, l.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &sep, &gw, &quit])?;

    TrayIconBuilder::new()
        .icon(tauri::image::Image::from_bytes(include_bytes!(
            "../icons/tray.png"
        ))?)
        .menu(&menu)
        .tooltip(DISPLAY_NAME)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match menu_action(event.id().as_ref()) {
            TrayMenuAction::Show => {
                restore_main_window(app);
            }
            TrayMenuAction::ToggleGateway => {
                let handle = app.clone();
                let background = app
                    .state::<crate::services::runtime_background::RuntimeBackgroundServices>()
                    .inner()
                    .clone();
                let _ = background.spawn_task(move |cancel| async move {
                    tokio::select! {
                        _ = cancel.cancelled() => {}
                        _ = async {
                            let gw = handle.state::<GatewayService>();
                            if gw.is_enabled().await {
                                if !gw.stop().await {
                                    ::log::warn!("[gateway] gateway-stop-timeout");
                                }
                            } else {
                                let config = gw.config().await;
                                let _ = gw.start(config, handle.clone()).await;
                            }
                        } => {}
                    }
                });
            }
            TrayMenuAction::Quit => {
                crate::app_exit::request(app, 0);
            }
            TrayMenuAction::Ignore => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if should_restore_from_tray_click(&button, &button_state) {
                    restore_main_window(tray.app_handle());
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{menu_action, should_restore_from_tray_click, TrayMenuAction};
    use tauri::tray::{MouseButton, MouseButtonState};

    #[test]
    fn quit_menu_is_a_true_quit_on_every_platform() {
        assert_eq!(menu_action("quit"), TrayMenuAction::Quit);
    }

    #[test]
    fn show_and_gateway_actions_stay_distinct_from_quit() {
        assert_eq!(menu_action("show"), TrayMenuAction::Show);
        assert_eq!(menu_action("gateway-toggle"), TrayMenuAction::ToggleGateway);
        assert_eq!(menu_action("unknown"), TrayMenuAction::Ignore);
    }

    #[test]
    fn tray_left_button_up_triggers_restore() {
        assert!(should_restore_from_tray_click(
            &MouseButton::Left,
            &MouseButtonState::Up
        ));
    }

    #[test]
    fn tray_not_restored_for_other_clicks() {
        assert!(!should_restore_from_tray_click(
            &MouseButton::Right,
            &MouseButtonState::Up
        ));
        assert!(!should_restore_from_tray_click(
            &MouseButton::Left,
            &MouseButtonState::Down
        ));
        assert!(!should_restore_from_tray_click(
            &MouseButton::Right,
            &MouseButtonState::Down
        ));
        assert!(!should_restore_from_tray_click(
            &MouseButton::Middle,
            &MouseButtonState::Up
        ));
        assert!(!should_restore_from_tray_click(
            &MouseButton::Middle,
            &MouseButtonState::Down
        ));
    }
}
