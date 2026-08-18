use crate::services::paths::data_dir;

pub fn initialize(app_handle: &tauri::AppHandle) -> Result<(), String> {
    run(app_handle)?;
    crate::services::private_store::repair_app_storage()
}

pub fn run(app_handle: &tauri::AppHandle) -> Result<(), String> {
    use std::fs;

    let new = data_dir();

    fs::create_dir_all(new.join("logs")).map_err(|_| migration_error())?;

    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir().ok_or_else(migration_error)?;

        let cl_go_legacy = home.join(".local/share/cl-go");
        let legacy_marker = new.join(".migrated-from-cl-go");
        if !legacy_marker.exists() && cl_go_legacy.exists() {
            crate::storage_migration_files::copy_items(&cl_go_legacy, &new)?;
            write_migration_file(&legacy_marker, b"ok")?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_support_wrong = dirs::data_local_dir().and_then(|d| {
            let p = d.join("cl-go-dash");
            if p != new {
                Some(p)
            } else {
                None
            }
        });
        let appsupport_marker = new.join(".migrated-from-appsupport");
        if let Some(wrong) = app_support_wrong {
            if !appsupport_marker.exists() && wrong.exists() {
                crate::storage_migration_files::copy_items(&wrong, &new)?;
                write_migration_file(&appsupport_marker, b"ok")?;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = dirs::data_dir().map(|d| d.join("cl-go-dash"));
        let win_marker = new.join(".migrated-from-appdata");
        if let Some(old) = appdata {
            if !win_marker.exists() && old.exists() {
                crate::storage_migration_files::copy_items(&old, &new)?;
                write_migration_file(&win_marker, b"ok")?;
            }
        }
    }

    init_base_structure(&new)?;
    crate::storage_default_skills::install_default_skills(app_handle, &new)?;
    crate::storage_migration_files::install_forecast_sidecar(app_handle, &new)?;

    Ok(())
}

fn init_base_structure(base: &std::path::Path) -> Result<(), String> {
    use std::fs;

    let dirs = [
        "memory/core",
        "inbox",
        "skills",
        "agent-sessions",
        "tool-results",
        "translations",
        "logs",
    ];
    for d in &dirs {
        fs::create_dir_all(base.join(d)).map_err(|_| migration_error())?;
    }

    let json_defaults: &[(&str, &str)] = &[
        ("config.json", "{}"),
        ("agent-settings.json", "{\"permissionMode\":\"auto\"}"),
        ("configured-providers.json", "[]"),
        ("favorite-models.json", "[]"),
        ("projects.json", "[]"),
        ("terminal-tabs.json", "[]"),
        ("inbox/pending.json", "[]"),
        (
            "personality-injection.json",
            "{\
                \"identity.md\":false,\
                \"principles.md\":false,\
                \"user.md\":false,\
                \"idea-discovery.md\":false\
            }",
        ),
    ];
    for (name, content) in json_defaults {
        let path = base.join(name);
        if !path.exists() {
            write_migration_file(&path, content.as_bytes())?;
        }
    }

    let empty_files = [
        "AGENTS.md",
        "memory/core/identity.md",
        "memory/core/principles.md",
        "memory/core/user.md",
        "inbox/idea-discovery.md",
    ];
    for name in &empty_files {
        let path = base.join(name);
        if !path.exists() {
            write_migration_file(&path, b"")?;
        }
    }

    Ok(())
}

fn write_migration_file(path: &std::path::Path, content: &[u8]) -> Result<(), String> {
    crate::services::private_store::atomic_write(path, content).map_err(|_| migration_error())
}

fn migration_error() -> String {
    "Erreur d'initialisation des données".to_string()
}

#[cfg(test)]
#[path = "storage_migration_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "storage_profile_compatibility_tests.rs"]
mod profile_compatibility_tests;
