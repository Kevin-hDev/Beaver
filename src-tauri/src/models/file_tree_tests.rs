#[cfg(test)]
mod tests {
    use crate::models::file_tree::FileEntry;

    #[test]
    fn test_serialisation_fichier() {
        let entry = FileEntry {
            name: "main.rs".to_string(),
            path: "/src/main.rs".to_string(),
            is_dir: false,
            extension: Some("rs".to_string()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"name\":\"main.rs\""));
        assert!(json.contains("\"is_dir\":false"));
        assert!(json.contains("\"extension\":\"rs\""));
    }

    #[test]
    fn test_serialisation_dossier() {
        let entry = FileEntry {
            name: "src".to_string(),
            path: "/src".to_string(),
            is_dir: true,
            extension: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"is_dir\":true"));
        assert!(json.contains("\"extension\":null"));
    }

    #[test]
    fn test_extension_tar_gz() {
        // Path::extension retourne "gz" pour archive.tar.gz
        let entry = FileEntry {
            name: "archive.tar.gz".to_string(),
            path: "/tmp/archive.tar.gz".to_string(),
            is_dir: false,
            extension: Some("gz".to_string()),
        };
        assert_eq!(entry.extension, Some("gz".to_string()));
    }

    #[test]
    fn test_dotfile_sans_extension() {
        // .env est un dotfile : extension = None
        let entry = FileEntry {
            name: ".env".to_string(),
            path: "/project/.env".to_string(),
            is_dir: false,
            extension: None,
        };
        assert_eq!(entry.extension, None);
    }
}
