use std::path::PathBuf;

/// Extrait les chemins de fichiers vus par un tool depuis son résultat textuel.
///
/// Les tools grep/glob/list_dir retournent du texte plain contenant des chemins
/// de fichiers. On parse ce texte pour les enregistrer comme "vus" dans le
/// WriteGuard, évitant le blocage injuste "fichier non lu avant écriture".
///
/// Retourne une liste de chemins (absolus quand possible).
pub fn extract_seen_paths(
    tool_name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    content: &str,
) -> Vec<PathBuf> {
    match tool_name {
        "grep" => extract_from_grep(content),
        "glob" => extract_from_glob(content),
        "list_dir" => extract_from_list_dir(args, working_dir, content),
        _ => Vec::new(),
    }
}

/// grep : chaque ligne de match est `chemin:ligne:texte`.
/// On prend la partie avant le 1er `:` comme chemin.
fn extract_from_grep(content: &str) -> Vec<PathBuf> {
    content
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("...") && !l.starts_with("(aucun"))
        .filter_map(|line| {
            // Format attendu : /chemin/absolu:fichier.rs:42:texte
            // Le chemin est avant le 1er `:`, mais attention au drive Windows (C:\).
            let path_part = if line.len() > 2 && line.as_bytes()[1] == b':' {
                // Windows : C:\... — on cherche le 3e `:` (après C:\chemin:ligne)
                let rest = &line[2..];
                match rest.find(':') {
                    Some(idx) => &line[..2 + idx],
                    None => return None,
                }
            } else {
                // Unix : chemin absolu ou relatif avant le 1er `:`
                match line.find(':') {
                    Some(idx) => &line[..idx],
                    None => return None,
                }
            };
            let p = std::path::Path::new(path_part);
            if p.is_file() {
                Some(p.to_path_buf())
            } else {
                None
            }
        })
        .collect()
}

/// glob : chaque ligne est un chemin absolu complet.
fn extract_from_glob(content: &str) -> Vec<PathBuf> {
    content
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("...") && !l.starts_with("(aucun"))
        .filter_map(|line| {
            let p = std::path::Path::new(line.trim());
            if p.is_file() {
                Some(p.to_path_buf())
            } else {
                None
            }
        })
        .collect()
}

/// list_dir : arbre indenté, noms relatifs au dossier racine.
/// On reconstruit le chemin absolu via le `path` d'entrée + l'indentation.
fn extract_from_list_dir(
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    content: &str,
) -> Vec<PathBuf> {
    let root_str = match args["path"].as_str() {
        Some(s) => s,
        None => return Vec::new(),
    };
    let root = std::path::Path::new(root_str);
    let base = if root.is_absolute() {
        root.to_path_buf()
    } else {
        working_dir.join(root)
    };

    // Suit la hiérarchie via l'indentation (2 espaces par niveau).
    // Les dossiers se terminent par `/`, les fichiers non.
    let mut stack: Vec<(usize, PathBuf)> = vec![(0, base.clone())];
    let mut files = Vec::new();

    for line in content.lines() {
        if line.is_empty() || line.starts_with("...") {
            continue;
        }
        let trimmed = line.trim_start_matches(' ');
        if trimmed.is_empty() || trimmed.starts_with('[') {
            continue;
        }
        let depth = (line.len() - trimmed.len()) / 2;
        let is_dir = trimmed.ends_with('/');
        let name = trimmed.trim_end_matches('/');

        // Remonte la stack jusqu'au bon parent.
        while stack.last().map(|(d, _)| *d >= depth).unwrap_or(false) {
            stack.pop();
        }
        let parent = stack
            .last()
            .map(|(_, p)| p.clone())
            .unwrap_or_else(|| base.clone());
        let full = parent.join(name);

        if is_dir {
            stack.push((depth, full));
        } else if full.is_file() {
            files.push(full);
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_grep_paths() {
        // Crée des fichiers réels pour que is_file() retourne true
        let dir = tempfile::tempdir().unwrap();
        let f1 = dir.path().join("main.rs");
        let f2 = dir.path().join("utils.rs");
        std::fs::write(&f1, "").unwrap();
        std::fs::write(&f2, "").unwrap();
        let content = format!(
            "{}:42:fn main() {{\n{}:10:pub fn helper()",
            f1.display(),
            f2.display()
        );
        let paths = extract_from_grep(&content);
        assert_eq!(paths.len(), 2);
        assert!(paths[0].to_string_lossy().ends_with("main.rs"));
    }

    #[test]
    fn parse_glob_paths() {
        let dir = tempfile::tempdir().unwrap();
        let f1 = dir.path().join("main.rs");
        let f2 = dir.path().join("utils.rs");
        std::fs::write(&f1, "").unwrap();
        std::fs::write(&f2, "").unwrap();
        let content = format!("{}\n{}", f1.display(), f2.display());
        let paths = extract_from_glob(&content);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn parse_list_dir_files() {
        // Crée des fichiers réels pour que is_file() retourne true
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("main.rs"), "").unwrap();
        std::fs::create_dir_all(root.join("utils")).unwrap();
        std::fs::write(root.join("utils").join("helper.rs"), "").unwrap();

        let content = "main.rs\nutils/\n  helper.rs";
        let args = json!({"path": root.to_str().unwrap()});
        let paths = extract_from_list_dir(&args, root, content);
        assert_eq!(paths.len(), 2, "devrait trouver main.rs + helper.rs");
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().ends_with("main.rs")));
        assert!(paths
            .iter()
            .any(|p| p.to_string_lossy().ends_with("helper.rs")));
    }

    #[test]
    fn ignores_truncation_lines() {
        let content = "... [tronqué à 100 résultats]\n(aucun résultat)\n... [3 erreur(s)]";
        assert!(extract_from_glob(content).is_empty());
        assert!(extract_from_grep(content).is_empty());
    }
}
