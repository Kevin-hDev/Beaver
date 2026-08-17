use tauri::menu::{Menu, MenuItem, MenuItemKind};

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

fn invalid_default_menu() -> tauri::Error {
    std::io::Error::other("macOS application menu layout is invalid").into()
}

pub(super) fn build(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::default(app)?;
    let app_submenu = menu
        .items()?
        .into_iter()
        .next()
        .and_then(|item| match item {
            MenuItemKind::Submenu(submenu) => Some(submenu),
            _ => None,
        })
        .ok_or_else(invalid_default_menu)?;
    let items = app_submenu.items()?;
    let last_position = items
        .len()
        .checked_sub(1)
        .ok_or_else(invalid_default_menu)?;
    let quit_text = match &items[last_position] {
        MenuItemKind::Predefined(item) if item.text()?.starts_with("Quit ") => item.text()?,
        _ => return Err(invalid_default_menu()),
    };

    // Tauri's predefined Quit calls Cocoa `terminate:` directly. Replacing only
    // that item preserves the standard menu while routing Quit through cleanup.
    app_submenu.remove_at(last_position)?;
    let quit = MenuItem::with_id(
        app,
        MACOS_QUIT_MENU_ID,
        quit_text,
        true,
        Some(MACOS_QUIT_ACCELERATOR),
    )?;
    app_submenu.append(&quit)?;
    Ok(menu)
}

pub(super) fn handle_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    if menu_action(event.id().as_ref()) == MacosMenuAction::CoordinatedQuit {
        crate::app_exit::request(app, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::{menu_action, MacosMenuAction, MACOS_QUIT_ACCELERATOR, MACOS_QUIT_MENU_ID};

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
    fn quit_uses_the_native_macos_accelerator() {
        assert_eq!(MACOS_QUIT_ACCELERATOR, "CmdOrCtrl+Q");
    }
}
