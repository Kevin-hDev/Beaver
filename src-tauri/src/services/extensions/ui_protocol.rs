use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::http::{header, Method, Request, Response, StatusCode};

const PROTOCOL: &str = "beaver-extension";
const MAX_FILE_BYTES: u64 = 4_194_304;
const MAIN_WEBVIEW: &str = "main";

#[derive(Clone)]
pub(crate) struct ProofArtifact {
    root: PathBuf,
    extension_id: String,
    manifest_sha: String,
}

impl ProofArtifact {
    #[cfg(any(test, feature = "e2e"))]
    fn new(root: PathBuf, extension_id: &str, manifest_sha: &str) -> Option<Self> {
        let root = root.canonicalize().ok()?;
        let metadata = std::fs::symlink_metadata(&root).ok()?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || !valid_id(extension_id)
            || !valid_sha(manifest_sha)
        {
            return None;
        }
        Some(Self {
            root,
            extension_id: extension_id.to_string(),
            manifest_sha: manifest_sha.to_string(),
        })
    }

    #[cfg(test)]
    pub(super) fn for_test(root: PathBuf, extension_id: &str, manifest_sha: &str) -> Self {
        Self::new(root, extension_id, manifest_sha).expect("valid UI protocol test artifact")
    }

    #[cfg(test)]
    pub(super) fn manifest_sha(&self) -> &str {
        &self.manifest_sha
    }
}

pub(crate) fn register<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    let artifact = proof_artifact_from_environment();
    builder.register_uri_scheme_protocol(PROTOCOL, move |context, request| {
        serve(context.webview_label(), &request, artifact.as_ref())
    })
}

pub(super) fn serve(
    webview_label: &str,
    request: &Request<Vec<u8>>,
    artifact: Option<&ProofArtifact>,
) -> Response<Vec<u8>> {
    if webview_label != MAIN_WEBVIEW {
        return empty(StatusCode::FORBIDDEN);
    }
    if request.method() != Method::GET {
        return empty(StatusCode::METHOD_NOT_ALLOWED);
    }
    let supplied_origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    // WebKit omits Origin for native custom-scheme module requests. The main
    // webview label remains authoritative; any supplied foreign origin closes.
    if supplied_origin.is_some_and(|value| !allowed_origin(value)) {
        return empty(StatusCode::FORBIDDEN);
    }
    let Some(artifact) = artifact else {
        return empty(StatusCode::NOT_FOUND);
    };
    let Some((extension_id, manifest_sha, name)) = request_parts(request) else {
        return empty(StatusCode::BAD_REQUEST);
    };
    let Some(origin) = supplied_origin.or_else(|| mapped_webview_origin(request)) else {
        return empty(StatusCode::FORBIDDEN);
    };
    if extension_id != artifact.extension_id || manifest_sha != artifact.manifest_sha {
        return empty(StatusCode::NOT_FOUND);
    }
    let Some(mime) = mime(name) else {
        return empty(StatusCode::NOT_FOUND);
    };
    let path = artifact
        .root
        .join(extension_id)
        .join(manifest_sha)
        .join(name);
    let Ok(metadata) = std::fs::symlink_metadata(&path) else {
        return empty(StatusCode::NOT_FOUND);
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return empty(StatusCode::NOT_FOUND);
    }
    if metadata.len() > MAX_FILE_BYTES {
        return empty(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let Ok(canonical) = path.canonicalize() else {
        return empty(StatusCode::NOT_FOUND);
    };
    if !canonical.starts_with(&artifact.root) {
        return empty(StatusCode::NOT_FOUND);
    }
    let Ok(file) = std::fs::File::open(&canonical) else {
        return empty(StatusCode::NOT_FOUND);
    };
    let mut body = Vec::with_capacity(metadata.len() as usize);
    if file
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut body)
        .is_err()
        || body.len() as u64 > MAX_FILE_BYTES
    {
        return empty(StatusCode::PAYLOAD_TOO_LARGE);
    }
    response(StatusCode::OK, body, Some((mime, origin)))
}

fn request_parts(request: &Request<Vec<u8>>) -> Option<(&str, &str, &str)> {
    let uri = request.uri();
    match (uri.scheme_str()?, uri.authority()?.as_str()) {
        (PROTOCOL, "localhost") | ("http", "beaver-extension.localhost") => {}
        _ => return None,
    }
    let path = uri.path();
    if path.contains(['%', '\\']) || path.split('/').any(|part| part == "..") {
        return None;
    }
    let parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();
    if parts.len() != 3 || !valid_id(parts[0]) || !valid_sha(parts[1]) || parts[2] != "entry.mjs" {
        return None;
    }
    Some((parts[0], parts[1], parts[2]))
}

fn allowed_origin(origin: &str) -> bool {
    matches!(origin, "tauri://localhost" | "http://tauri.localhost")
        || cfg!(debug_assertions) && origin == "http://localhost:5173"
}

fn mapped_webview_origin(request: &Request<Vec<u8>>) -> Option<&'static str> {
    match (
        request.uri().scheme_str()?,
        request.uri().authority()?.as_str(),
    ) {
        (PROTOCOL, "localhost") => Some("tauri://localhost"),
        ("http", "beaver-extension.localhost") => Some("http://tauri.localhost"),
        _ => None,
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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

fn empty(status: StatusCode) -> Response<Vec<u8>> {
    response(status, Vec::new(), None)
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

#[cfg(feature = "e2e")]
fn proof_artifact_from_environment() -> Option<ProofArtifact> {
    let profile = std::env::var_os("CL_GO_CEF_TEST_DATA_DIR")?;
    let manifest_sha = std::env::var("BEAVER_E2E_UI_MANIFEST_SHA").ok()?;
    ProofArtifact::new(
        PathBuf::from(profile).join("extensions-ui-proof"),
        "ui-proof",
        &manifest_sha,
    )
}

#[cfg(not(feature = "e2e"))]
fn proof_artifact_from_environment() -> Option<ProofArtifact> {
    None
}
