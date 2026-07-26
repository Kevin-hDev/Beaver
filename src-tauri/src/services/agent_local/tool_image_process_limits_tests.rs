#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_image_process::process_image;
    use tempfile::TempDir;

    fn working_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn create_test_image(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let img = image::RgbImage::new(50, 50);
        img.save(&path).unwrap();
        path
    }

    #[tokio::test]
    async fn resize_too_large_returns_error() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png");
        let output = tmp.path().join("output.png");
        let ops =
            serde_json::json!([{ "type": "resize", "width": 9000, "height": 50, "mode": "exact" }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn crop_out_of_bounds_returns_error() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png");
        let output = tmp.path().join("output.png");
        let ops =
            serde_json::json!([{ "type": "crop", "x": 40, "y": 40, "width": 20, "height": 20 }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn webp_quality_returns_warning() {
        let tmp = working_dir();
        let input = create_test_image(tmp.path(), "input.png");
        let output = tmp.path().join("output.webp");
        let ops = serde_json::json!([{ "type": "quality", "value": 50 }]);
        let result = process_image(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &ops,
            tmp.path(),
        )
        .await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert!(json["warning"].as_str().unwrap_or("").contains("WebP"));
    }
}
