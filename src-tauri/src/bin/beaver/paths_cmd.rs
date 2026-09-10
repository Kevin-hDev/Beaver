use crate::output::Out;
use std::path::{Path, PathBuf};

type KnownEntry = (&'static str, &'static str, PathBuf);

pub fn known_entries(root: &Path) -> Vec<KnownEntry> {
    vec![
        ("Dossier des données", "Data directory", root.to_path_buf()),
        ("Configuration", "Configuration", root.join("config.json")),
        ("Coffre", "Vault", root.join("secrets.enc")),
        (
            "Conversations",
            "Conversations",
            root.join("agent-sessions"),
        ),
        ("Journaux", "Logs", root.join("logs")),
        ("Projets", "Projects", root.join("projects.json")),
        ("Skills", "Skills", root.join("skills")),
        (
            "Résultats d'outils",
            "Tool results",
            root.join("tool-results"),
        ),
        (
            "Connecteurs MCP",
            "MCP connectors",
            root.join("mcp-connectors.json"),
        ),
        (
            "Moteur Ollama",
            "Ollama engine",
            cl_go_dash_lib::cli_support::ollama_bundle_dir(root),
        ),
    ]
}

pub fn run(out: &Out) -> i32 {
    let root = cl_go_dash_lib::cli_support::data_dir();
    for (fr, en, path) in known_entries(&root) {
        let suffix = if path.exists() {
            ""
        } else {
            out.t(" (pas encore créé)", " (not created yet)")
        };
        out.line(&format!("{} : {}{suffix}", out.t(fr, en), path.display()));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_entrees_sont_toutes_sous_la_racine() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        for (_, _, path) in known_entries(directory.path()) {
            assert!(
                path.starts_with(directory.path()),
                "{} hors de la racine",
                path.display()
            );
        }
    }

    #[test]
    fn les_entrees_couvrent_les_essentiels() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let paths = known_entries(directory.path())
            .into_iter()
            .map(|(_, _, path)| path)
            .collect::<Vec<_>>();
        for expected in ["config.json", "secrets.enc", "logs", "agent-sessions"] {
            assert!(
                paths.iter().any(|path| path.ends_with(expected)),
                "manque {expected}"
            );
        }
    }

    #[test]
    fn le_bundle_ollama_vient_de_l_autorite_des_chemins() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let entry = known_entries(directory.path())
            .into_iter()
            .find(|(_, en, _)| *en == "Ollama engine")
            .expect("Ollama entry");
        assert_eq!(entry.2, directory.path().join("ollama-bundle"));
    }
}
