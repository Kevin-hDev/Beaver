use super::DetectedEditor;
use std::path::Path;
use std::process::Command;

fn safe_extension(file_path: &Path) -> Option<String> {
    let ext = file_path.extension()?.to_str()?;
    if ext.is_empty() || ext.len() > 20 || !ext.chars().all(|c| c.is_alphanumeric()) {
        return None;
    }
    Some(format!(".{ext}"))
}

pub fn detect(file_path: &Path) -> Vec<DetectedEditor> {
    let ext = match safe_extension(file_path) {
        Some(e) => e,
        None => return vec![],
    };

    let mut editors = Vec::new();

    let assoc_out = Command::new("cmd").args(["/C", "assoc"]).arg(&ext).output();

    if let Ok(out) = assoc_out {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(first_line) = text.lines().next() {
            if let Some(ftype) = first_line.split('=').nth(1) {
                let ftype = ftype.trim();
                if !ftype.is_empty() {
                    editors.push(DetectedEditor {
                        name: ftype.to_string(),
                        path: ftype.to_string(),
                        is_default: true,
                    });
                }
            }
        }
    }

    let known = [
        ("Notepad", "C:\\Windows\\notepad.exe"),
        (
            "Visual Studio Code",
            "C:\\Program Files\\Microsoft VS Code\\Code.exe",
        ),
    ];
    for (label, app_path) in known {
        if Path::new(app_path).is_file() {
            editors.push(DetectedEditor {
                name: label.to_string(),
                path: app_path.to_string(),
                is_default: false,
            });
        }
    }

    editors
}
