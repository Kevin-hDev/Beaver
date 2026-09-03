use std::path::PathBuf;
use subtle::ConstantTimeEq;
use tauri::http::{header, Method, Request, Response};

pub(crate) use super::ui_protocol_proof::ProofArtifact;
use super::ui_protocol_response::{not_found, read_response};

const PROTOCOL: &str = "beaver-extension";
const MAIN_WEBVIEW: &str = "main";

struct GrantedArtifact {
    root: PathBuf,
    output_names: Vec<String>,
}

pub(crate) fn register<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    startup: super::UiStartupState,
) -> tauri::Builder<R> {
    let proof = super::ui_protocol_proof::from_environment();
    builder.register_uri_scheme_protocol(PROTOCOL, move |context, request| {
        serve_request(context.webview_label(), &request, &startup, proof.as_ref())
    })
}

#[cfg(test)]
pub(super) fn serve(
    webview_label: &str,
    request: &Request<Vec<u8>>,
    proof: Option<&ProofArtifact>,
) -> Response<Vec<u8>> {
    serve_request(
        webview_label,
        request,
        &super::UiStartupState::resolved(super::UiStartupMode::Normal),
        proof,
    )
}

#[cfg(test)]
pub(super) fn serve_with_startup(
    webview_label: &str,
    request: &Request<Vec<u8>>,
    startup: &super::UiStartupState,
    proof: Option<&ProofArtifact>,
) -> Response<Vec<u8>> {
    serve_request(webview_label, request, startup, proof)
}

fn serve_request(
    webview_label: &str,
    request: &Request<Vec<u8>>,
    startup: &super::UiStartupState,
    proof: Option<&ProofArtifact>,
) -> Response<Vec<u8>> {
    let Some((extension_id, manifest_sha, name)) = request_parts(request) else {
        return not_found();
    };
    let supplied_origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if webview_label != MAIN_WEBVIEW
        || request.method() != Method::GET
        || request.uri().query().is_some()
        || supplied_origin.is_some_and(|origin| !allowed_origin(origin))
        || !startup.protocol_loading_allowed_for(extension_id)
    {
        return not_found();
    }
    let Some(origin) = supplied_origin.or_else(|| mapped_webview_origin(request)) else {
        return not_found();
    };
    let grant = proof
        .and_then(|artifact| proof_grant(artifact, extension_id, manifest_sha))
        .or_else(|| active_grant(extension_id, manifest_sha));
    let Some(grant) = grant else {
        return not_found();
    };
    if !grant.output_names.iter().any(|output| output == name) {
        return not_found();
    }
    read_response(&grant.root, name, origin).unwrap_or_else(not_found)
}

fn active_grant(extension_id: &str, manifest_sha: &str) -> Option<GrantedArtifact> {
    let record = super::registry::find(extension_id).ok()?;
    let ui = record.manifest.ui.as_ref()?;
    let artifact = record.ui_artifact.as_ref()?;
    if record.kind != super::types::ExtensionKind::Local
        || !record.enabled
        || !record.trusted
        || ui.mode != super::types::ExtensionUiMode::Advanced
        || !constant_sha(manifest_sha, &artifact.manifest_sha256)
        || !super::fingerprint::is_current(&record).ok()?
    {
        return None;
    }
    let root = super::ui_artifact_store::artifact_path(extension_id, manifest_sha).ok()?;
    super::ui_artifact::verify_at(&root, artifact).ok()?;
    Some(GrantedArtifact {
        root,
        output_names: artifact
            .outputs
            .iter()
            .map(|output| output.name.clone())
            .collect(),
    })
}

fn proof_grant(
    proof: &ProofArtifact,
    extension_id: &str,
    manifest_sha: &str,
) -> Option<GrantedArtifact> {
    if proof.extension_id != extension_id || !constant_sha(&proof.manifest_sha, manifest_sha) {
        return None;
    }
    Some(GrantedArtifact {
        root: proof.root().join(extension_id).join(manifest_sha),
        output_names: vec!["entry.mjs".to_string()],
    })
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
    if parts.len() != 3
        || super::validation::identifier(parts[0]).is_err()
        || !valid_sha(parts[1])
        || !valid_name(parts[2])
    {
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

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(super) fn valid_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn constant_sha(left: &str, right: &str) -> bool {
    bool::from(left.as_bytes().ct_eq(right.as_bytes()))
}
