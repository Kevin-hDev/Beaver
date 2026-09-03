use std::io::Read;
use std::path::Path;
use tauri::http::{header, Response, StatusCode};

pub(super) fn read_response(root: &Path, name: &str, origin: &str) -> Option<Response<Vec<u8>>> {
    let mime = mime(name)?;
    let path = root.join(name);
    let metadata = std::fs::symlink_metadata(&path).ok()?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > super::ui_contract::MAX_ADVANCED_ARTIFACT_BYTES as u64
    {
        return None;
    }
    let root = dunce::canonicalize(root).ok()?;
    let canonical = dunce::canonicalize(&path).ok()?;
    if !canonical.starts_with(&root) {
        return None;
    }
    let file = std::fs::File::open(canonical).ok()?;
    let mut body = Vec::with_capacity(metadata.len() as usize);
    file.take(super::ui_contract::MAX_ADVANCED_ARTIFACT_BYTES as u64 + 1)
        .read_to_end(&mut body)
        .ok()?;
    if body.len() as u64 != metadata.len() {
        return None;
    }
    Some(response(StatusCode::OK, body, Some((mime, origin))))
}

pub(super) fn not_found() -> Response<Vec<u8>> {
    response(StatusCode::NOT_FOUND, Vec::new(), None)
}

fn mime(name: &str) -> Option<&'static str> {
    match Path::new(name).extension()?.to_str()? {
        "mjs" | "js" => Some("text/javascript; charset=utf-8"),
        "css" => Some("text/css; charset=utf-8"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "woff2" => Some("font/woff2"),
        _ => None,
    }
}

fn response(status: StatusCode, body: Vec<u8>, success: Option<(&str, &str)>) -> Response<Vec<u8>> {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CACHE_CONTROL, "no-store")
        .header("X-Content-Type-Options", "nosniff");
    if let Some((mime, origin)) = success {
        builder = builder
            .header(header::CONTENT_TYPE, mime)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
            .header(header::VARY, "Origin");
    }
    builder
        .body(body)
        .unwrap_or_else(|_| Response::new(Vec::new()))
}
