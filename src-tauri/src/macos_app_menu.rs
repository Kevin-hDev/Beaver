use tauri::menu::{
    AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID,
};

pub(super) const MACOS_QUIT_MENU_ID: &str = "beaver-coordinated-quit";
const MACOS_QUIT_ACCELERATOR: &str = "CmdOrCtrl+Q";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacosMenuAction {
    CoordinatedQuit,
    Ignore,
}

pub(super) fn menu_action(id: &str) -> MacosMenuAction {
    if id == MACOS_QUIT_MENU_ID {
        MacosMenuAction::CoordinatedQuit
    } else {
        MacosMenuAction::Ignore
    }
}

pub(super) fn build(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let package = app.package_info();
    let about = AboutMetadata {
        name: Some(package.name.clone()),
        version: Some(package.version.to_string()),
        copyright: app.config().bundle.copyright.clone(),
        authors: app
            .config()
            .bundle
            .publisher
            .clone()
            .map(|value| vec![value]),
        ..Default::default()
    };
    let native_quit = PredefinedMenuItem::quit(app, None)?;
    let quit_text = native_quit.text()?;
    let quit = MenuItem::with_id(
        app,
        MACOS_QUIT_MENU_ID,
        quit_text,
        true,
        Some(MACOS_QUIT_ACCELERATOR),
    )?;

    // Build from public primitives: the framework's default item order and
    // localized Quit label shape are not stable contracts.
    let app_menu = Submenu::with_items(
        app,
        package.name.clone(),
        true,
        &[
            &PredefinedMenuItem::about(app, None, Some(about))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::services(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[&PredefinedMenuItem::close_window(app, None)?],
    )?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[&PredefinedMenuItem::fullscreen(app, None)?],
    )?;
    let window_menu = Submenu::with_id_and_items(
        app,
        WINDOW_SUBMENU_ID,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::maximize(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    let help_menu = Submenu::with_id_and_items(app, HELP_SUBMENU_ID, "Help", true, &[])?;
    Menu::with_items(
        app,
        &[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &window_menu,
            &help_menu,
        ],
    )
}

pub(super) fn handle_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    if menu_action(event.id().as_ref()) == MacosMenuAction::CoordinatedQuit {
        crate::app_exit::request(app, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::{menu_action, MacosMenuAction, MACOS_QUIT_MENU_ID};

    #[test]
    fn only_the_owned_quit_item_requests_application_exit() {
        assert_eq!(
            menu_action(MACOS_QUIT_MENU_ID),
            MacosMenuAction::CoordinatedQuit
        );
        assert_eq!(menu_action("quit"), MacosMenuAction::Ignore);
        assert_eq!(menu_action("unknown"), MacosMenuAction::Ignore);
    }

    #[test]
    fn menu_does_not_depend_on_default_layout_or_localized_text_shape() {
        let source = include_str!("macos_app_menu.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source must precede tests");

        assert!(!production.contains(&["Menu", "::default"].concat()));
        assert!(!production.contains(&["starts", "_with"].concat()));
        assert_eq!(production.matches("native_quit.text()?").count(), 1);
        assert!(production.contains("&quit,"));
        assert!(production.contains("WINDOW_SUBMENU_ID"));
        assert!(production.contains("HELP_SUBMENU_ID"));
    }
}
