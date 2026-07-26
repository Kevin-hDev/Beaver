use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn data_root() -> PathBuf {
    crate::services::paths::data_dir()
}

fn state_path() -> PathBuf {
    data_root().join("personality-injection.json")
}

pub fn read_state() -> HashMap<String, bool> {
    let path = state_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn write_state(state: &HashMap<String, bool>) -> Result<(), String> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create dir: {}", e))?;
    }
    let tmp = path.with_extension("json.tmp");
    let content =
        serde_json::to_string_pretty(state).map_err(|e| format!("Cannot serialize: {}", e))?;
    fs::write(&tmp, &content).map_err(|e| format!("Cannot write: {}", e))?;
    fs::rename(&tmp, &path).map_err(|e| format!("Cannot rename: {}", e))?;
    Ok(())
}

pub fn load_injected_contents() -> Option<String> {
    let state = read_state();
    let root = data_root();
    let core = root.join("memory/core");
    let inbox = root.join("inbox");

    let mut sections: Vec<String> = Vec::new();

    for (name, enabled) in &state {
        if !enabled {
            continue;
        }
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            continue;
        }
        let path = core.join(name);
        let path = if path.exists() {
            path
        } else {
            let alt = inbox.join(name);
            if alt.exists() {
                alt
            } else {
                continue;
            }
        };
        if let Ok(content) = fs::read_to_string(&path) {
            let content = content.trim();
            if !content.is_empty() {
                sections.push(format!(
                    "Contents of {} (personality context):\n\n{}",
                    path.display(),
                    content,
                ));
            }
        }
    }

    if sections.is_empty() {
        return None;
    }
    Some(sections.join("\n\n"))
}
