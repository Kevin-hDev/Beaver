pub const PROTECTED_APP_FILES: &[&str] = &[
    "config.json",
    "secrets.enc",
    "agent-settings.json",
    "configured-providers.json",
];

const SENSITIVE_PATH_MARKERS: &[&str] = &[
    ".env",
    ".ssh/",
    "/.ssh",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    ".npmrc",
    ".pypirc",
    ".netrc",
    ".aws/credentials",
    ".config/gcloud",
    ".kube/config",
    "credentials",
    "login.keychain",
    "keychain-db",
];

pub fn bash_touches_sensitive_data(command: &str) -> bool {
    // On retire le contenu des heredocs avant l'analyse : un fichier qui
    // mentionne `.env` (un .gitignore par ex.) ne doit pas déclencher l'alerte,
    // seul un vrai chemin `.env` en argument de commande compte.
    let without_heredoc = strip_heredoc_bodies(command);
    let normalized = without_heredoc.replace('\\', "/").to_lowercase();
    SENSITIVE_PATH_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || mentions_protected_app_file(&normalized)
}

pub fn is_sensitive_path(path: &std::path::Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    SENSITIVE_PATH_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || mentions_protected_app_file(&normalized)
}

/// Retire le corps des heredocs (`<<'EOF' ... EOF` ou `<<EOF ... EOF`) d'une
/// commande shell. Conserve tout le reste (chemins cibles, redirections).
fn strip_heredoc_bodies(command: &str) -> String {
    let mut out = String::with_capacity(command.len());
    let mut delimiter: Option<String> = None;
    for line in command.lines() {
        if delimiter.is_none() {
            if let Some(delim) = heredoc_delimiter(line) {
                // On garde la ligne d'ouverture (contient le chemin/redirect) puis
                // on entre en mode "corps" : les lignes suivantes sont du contenu.
                out.push_str(line);
                out.push('\n');
                delimiter = Some(delim);
            } else {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }
        // En mode corps : on cherche la ligne fermante (le délimiteur seul).
        if line.trim() == delimiter.as_deref().unwrap_or("") {
            delimiter = None;
            out.push_str(line);
            out.push('\n');
        }
        // Sinon on ignore la ligne (contenu du heredoc).
    }
    out
}

/// Extrait le nom du délimiteur d'heredoc depuis une ligne d'ouverture.
/// Gère les formes : `<< EOF`, `<<'EOF'`, `<<"EOF"`, `<<- 'EOF'`, etc.
/// Renvoie `None` si la ligne n'ouvre pas de heredoc.
fn heredoc_delimiter(line: &str) -> Option<String> {
    let l = line.trim();
    let after = l.split("<<").nth(1)?;
    let cleaned: String = after
        .trim()
        .trim_start_matches('-')
        .trim()
        .trim_matches(|c: char| c == '\'' || c == '"')
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if cleaned.len() >= 2 {
        Some(cleaned)
    } else {
        None
    }
}

fn mentions_protected_app_file(normalized_command: &str) -> bool {
    let data_dir = crate::services::paths::data_dir()
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();
    PROTECTED_APP_FILES.iter().any(|file| {
        let full_path = format!("{data_dir}/{file}");
        let app_relative = format!(".local/share/cl-go-dash/{file}");
        let app_folder = format!("cl-go-dash/{file}");
        normalized_command.contains(&full_path)
            || normalized_command.contains(&app_relative)
            || normalized_command.contains(&app_folder)
    })
}

include!("sensitive_data_redaction.rs");

#[cfg(test)]
#[path = "sensitive_data_tests.rs"]
mod tests;
