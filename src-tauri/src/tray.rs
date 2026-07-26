use crate::services::{brand::DISPLAY_NAME, gateway::GatewayService};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

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

pub fn create_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let l = labels();
    let show = MenuItem::with_id(app, "show", l.show, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let gw = MenuItem::with_id(app, "gateway-toggle", l.gateway, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", l.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &sep, &gw, &quit])?;

    TrayIconBuilder::new()
        .icon(tauri::image::Image::from_bytes(include_bytes!(
            "../icons/tray.png"
        ))?)
        .menu(&menu)
        .tooltip(DISPLAY_NAME)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "gateway-toggle" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let gw = handle.state::<GatewayService>();
                    if gw.is_enabled().await {
                        gw.stop().await;
                    } else {
                        let config = gw.config().await;
                        let _ = gw.start(config, handle.clone()).await;
                    }
                });
            }
            "quit" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let gw = handle.state::<GatewayService>();
                    gw.stop().await;
                    handle.exit(0);
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { .. } = event {
                if let Some(win) = tray.app_handle().get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}
