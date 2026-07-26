#[cfg(test)]
mod tests {
    use crate::commands::file_tree::{list_directory, HIDDEN_ENTRIES};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tree() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        fs::create_dir(root.join("node_modules")).unwrap();
        fs::write(root.join("README.md"), "doc").unwrap();
        fs::write(root.join(".env"), "SECRET=x").unwrap();
        fs::write(root.join(".DS_Store"), "").unwrap();

        tmp
    }

    #[tokio::test]
    async fn test_tri_dossiers_dabord_puis_fichiers_alpha() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir(root.join("zeta")).unwrap();
        fs::create_dir(root.join("alpha")).unwrap();
        fs::write(root.join("main.rs"), "").unwrap();
        fs::write(root.join("build.rs"), "").unwrap();

        let entries = list_directory(root.to_str().unwrap().to_string(), false, None)
            .await
            .unwrap();

        assert_eq!(entries[0].name, "alpha");
        assert!(entries[0].is_dir);
        assert_eq!(entries[1].name, "zeta");
        assert!(entries[1].is_dir);
        assert_eq!(entries[2].name, "build.rs");
        assert!(!entries[2].is_dir);
        assert_eq!(entries[3].name, "main.rs");
    }

    #[tokio::test]
    async fn test_hidden_masque_git_et_dsstore_mais_pas_node_modules() {
        let tmp = create_test_tree();
        let root = tmp.path();

        let entries = list_directory(root.to_str().unwrap().to_string(), false, None)
            .await
            .unwrap();

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

        assert!(!names.contains(&".git"), ".git doit être masqué");
        assert!(!names.contains(&".DS_Store"), ".DS_Store doit être masqué");
        assert!(
            names.contains(&"node_modules"),
            "node_modules doit être visible"
        );
        assert!(names.contains(&".env"), ".env doit être visible");
        assert!(names.contains(&"README.md"), "README.md doit être visible");
    }

    #[tokio::test]
    async fn test_show_hidden_true_montre_git() {
        let tmp = create_test_tree();
        let root = tmp.path();

        let entries = list_directory(root.to_str().unwrap().to_string(), true, None)
            .await
            .unwrap();

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.contains(&".git"),
            ".git doit apparaître en mode show_hidden"
        );
        assert!(
            names.contains(&".DS_Store"),
            ".DS_Store doit apparaître en mode show_hidden"
        );
    }

    #[tokio::test]
    async fn test_dossier_inexistant_retourne_erreur() {
        let result = list_directory(
            "/tmp/dossier_qui_nexiste_vraiment_pas_xyz123".to_string(),
            false,
            None,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Dossier introuvable");
    }

    #[tokio::test]
    async fn test_path_traversal_retourne_erreur() {
        let result = list_directory("../../../etc".to_string(), false, None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Chemin invalide");
    }

    #[tokio::test]
    async fn test_path_vide_retourne_erreur() {
        let result = list_directory("".to_string(), false, None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Chemin invalide");
    }

    #[tokio::test]
    async fn test_path_avec_null_byte_retourne_erreur() {
        let result = list_directory("/tmp/foo\0bar".to_string(), false, None).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Chemin invalide");
    }

    #[test]
    fn test_hidden_entries_contient_les_bons_elements() {
        assert!(HIDDEN_ENTRIES.contains(&".git"));
        assert!(HIDDEN_ENTRIES.contains(&".DS_Store"));
        assert!(HIDDEN_ENTRIES.contains(&".next"));
        assert!(HIDDEN_ENTRIES.contains(&".turbo"));
        assert!(HIDDEN_ENTRIES.contains(&"__pycache__"));
        assert!(HIDDEN_ENTRIES.contains(&"dist"));
        assert!(HIDDEN_ENTRIES.contains(&"target"));
        assert!(HIDDEN_ENTRIES.contains(&"build"));
        assert!(HIDDEN_ENTRIES.contains(&".cache"));
    }

    #[tokio::test]
    async fn test_extension_extraite_correctement() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::write(root.join("main.rs"), "").unwrap();
        fs::write(root.join("archive.tar.gz"), "").unwrap();
        fs::write(root.join(".env"), "").unwrap();
        fs::write(root.join("Makefile"), "").unwrap();

        let entries = list_directory(root.to_str().unwrap().to_string(), true, None)
            .await
            .unwrap();

        let find = |name: &str| -> Option<&crate::models::file_tree::FileEntry> {
            entries.iter().find(|e| e.name == name)
        };

        assert_eq!(find("main.rs").unwrap().extension, Some("rs".to_string()));
        assert_eq!(
            find("archive.tar.gz").unwrap().extension,
            Some("gz".to_string())
        );
        assert_eq!(find(".env").unwrap().extension, None);
    }

    #[tokio::test]
    async fn test_confinement_projet_bloque_chemin_hors_racine() {
        let project = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let root = project.path();

        fs::create_dir(root.join("src")).unwrap();

        let result = list_directory(
            outside.path().to_str().unwrap().to_string(),
            false,
            Some(root.to_str().unwrap().to_string()),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Chemin invalide");
    }

    #[tokio::test]
    async fn test_confinement_projet_autorise_sous_dossier() {
        let project = TempDir::new().unwrap();
        let root = project.path();

        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src").join("main.rs"), "").unwrap();

        let entries = list_directory(
            root.join("src").to_str().unwrap().to_string(),
            false,
            Some(root.to_str().unwrap().to_string()),
        )
        .await
        .unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "main.rs");
    }
}
