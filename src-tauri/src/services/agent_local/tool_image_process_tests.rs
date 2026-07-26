#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_image_process::process_image;
    use tempfile::TempDir;

    fn working_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn create_test_image(dir: &std::path::Path, name: &str, w: u32, h: u32) -> std::path::PathBuf {
        let path = dir.join(name);
        let img = image::RgbImage::new(w, h);
        img.save(&path).unwrap();
        path
    }

    #[tokio::test]
    async fn resize_fit() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 100, 200);
        let output = tmp.path().join("output.png");
        let ops =
            serde_json::json!([{ "type": "resize", "width": 50, "height": 50, "mode": "fit" }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        // fit 100x200 into 50x50 → 25x50 (ratio preserved)
        assert_eq!(json["width"], 25);
        assert_eq!(json["height"], 50);
    }

    #[tokio::test]
    async fn resize_exact() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 100, 200);
        let output = tmp.path().join("output.png");
        let ops =
            serde_json::json!([{ "type": "resize", "width": 50, "height": 50, "mode": "exact" }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["width"], 50);
        assert_eq!(json["height"], 50);
    }

    #[tokio::test]
    async fn resize_fill() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 100, 200);
        let output = tmp.path().join("output.png");
        let ops =
            serde_json::json!([{ "type": "resize", "width": 50, "height": 50, "mode": "fill" }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["width"], 50);
        assert_eq!(json["height"], 50);
    }

    #[tokio::test]
    async fn crop_image() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 100, 100);
        let output = tmp.path().join("output.png");
        let ops =
            serde_json::json!([{ "type": "crop", "x": 10, "y": 10, "width": 50, "height": 30 }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["width"], 50);
        assert_eq!(json["height"], 30);
    }

    #[tokio::test]
    async fn convert_png_to_jpeg() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 50, 50);
        let output = tmp.path().join("output.jpg");
        let ops = serde_json::json!([]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error);
        assert!(output.exists());
    }

    #[tokio::test]
    async fn invalid_input_path() {
        let tmp = working_dir();
        let output = tmp.path().join("output.png");
        let ops = serde_json::json!([]);
        let result = process_image(
            "/nonexistent/image.png",
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn no_operations_preserves_dimensions() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 80, 60);
        let output = tmp.path().join("output.png");
        let ops = serde_json::json!([]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["width"], 80);
        assert_eq!(json["height"], 60);
    }

    #[tokio::test]
    async fn result_contains_file_size() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 50, 50);
        let output = tmp.path().join("output.png");
        let ops = serde_json::json!([]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert!(json["file_size_bytes"].as_u64().unwrap_or(0) > 0);
    }

    #[tokio::test]
    async fn unknown_operation_returns_error() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png", 50, 50);
        let output = tmp.path().join("output.png");
        let ops = serde_json::json!([{ "type": "unknown_op" }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(result.is_error);
    }
}
