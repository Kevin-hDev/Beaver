#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_image_read::read_image;
    use tempfile::TempDir;

    fn working_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[tokio::test]
    async fn read_png_metadata() {
        let tmp = working_dir();
        let img_path = tmp.path().join("test.png");
        let img = image::RgbImage::new(2, 2);
        img.save(&img_path).unwrap();

        let result = read_image(img_path.to_str().unwrap(), tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["width"], 2);
        assert_eq!(json["height"], 2);
        assert_eq!(json["format"], "png");
        assert!(json["file_size_bytes"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn read_png_larger() {
        let tmp = working_dir();
        let img_path = tmp.path().join("large.png");
        let img = image::RgbImage::new(100, 80);
        img.save(&img_path).unwrap();

        let result = read_image(img_path.to_str().unwrap(), tmp.path()).await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["width"], 100);
        assert_eq!(json["height"], 80);
    }

    #[tokio::test]
    async fn read_invalid_path() {
        let tmp = working_dir();
        let result = read_image("/nonexistent/image.png", tmp.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn read_unsupported_format() {
        let tmp = working_dir();
        let path = tmp.path().join("test.svg");
        std::fs::write(&path, "<svg></svg>").unwrap();
        let result = read_image(path.to_str().unwrap(), tmp.path()).await;
        assert!(result.is_error);
        assert!(
            result.content.contains("Format non supporté"),
            "content: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_empty_path() {
        let tmp = working_dir();
        let result = read_image("", tmp.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn jpg_normalizes_to_jpeg() {
        let tmp = working_dir();
        let img_path = tmp.path().join("test.jpg");
        let img = image::RgbImage::new(4, 4);
        img.save(&img_path).unwrap();

        let result = read_image(img_path.to_str().unwrap(), tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["format"], "jpeg");
    }
}
