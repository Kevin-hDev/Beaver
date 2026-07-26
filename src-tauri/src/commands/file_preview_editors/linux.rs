use super::DetectedEditor;
use std::path::Path;
use std::process::Command;

fn get_mime_type(file_path: &Path) -> Option<String> {
    let output = Command::new("xdg-mime")
        .args(["query", "filetype", file_path.to_str()?])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn get_default_app(mime: &str) -> Option<String> {
    let output = Command::new("xdg-mime")
        .args(["query", "default", mime])
        .output()
        .ok()?;
    if output.status.success() {
        let desktop = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !desktop.is_empty() {
            Some(desktop)
        } else {
            None
        }
    } else {
        None
    }
}

fn desktop_name(desktop_id: &str) -> String {
    let search_dirs = ["/usr/share/applications", "/usr/local/share/applications"];
    let home = std::env::var("HOME").unwrap_or_default();
    let user_dir = format!("{home}/.local/share/applications");

    for dir in search_dirs
        .iter()
        .chain(std::iter::once(&user_dir.as_str()))
    {
        let path = Path::new(dir).join(desktop_id);
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                if let Some(name) = line.strip_prefix("Name=") {
                    return name.to_string();
                }
            }
        }
    }
    desktop_id.trim_end_matches(".desktop").to_string()
}

fn apps_for_mime(mime: &str) -> Vec<String> {
    let search_dirs = ["/usr/share/applications", "/usr/local/share/applications"];
    let home = std::env::var("HOME").unwrap_or_default();
    let user_dir = format!("{home}/.local/share/applications");
    let mut result = Vec::new();

    for dir in search_dirs
        .iter()
        .chain(std::iter::once(&user_dir.as_str()))
    {
        let cache = Path::new(dir).join("mimeinfo.cache");
        if let Ok(content) = std::fs::read_to_string(&cache) {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix(&format!("{mime}=")) {
                    for desktop in rest.split(';') {
                        let d = desktop.trim();
                        if !d.is_empty() && !result.contains(&d.to_string()) {
                            result.push(d.to_string());
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn detect(file_path: &Path) -> Vec<DetectedEditor> {
    let mime = match get_mime_type(file_path) {
        Some(m) => m,
        None => return vec![],
    };

    let default_desktop = get_default_app(&mime);
    let all_desktops = apps_for_mime(&mime);
    let mut editors = Vec::new();

    for desktop_id in &all_desktops {
        let is_default = default_desktop.as_ref() == Some(desktop_id);
        editors.push(DetectedEditor {
            name: desktop_name(desktop_id),
            path: desktop_id.clone(),
            is_default,
        });
    }

    editors.sort_by_key(|b| std::cmp::Reverse(b.is_default));
    editors
}
