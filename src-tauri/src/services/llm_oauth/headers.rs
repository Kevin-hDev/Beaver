use std::io::Read;
use std::path::Path;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT};

use super::LlmOAuthProvider;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::{brand, paths, private_store};

pub const KIMI_PLATFORM_HEADER: &str = "kimi_code_cli";
const MAX_DEVICE_ID_BYTES: u64 = 64;

pub fn request_headers(provider: LlmOAuthProvider) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    insert(&mut headers, USER_AGENT, &user_agent())?;
    if provider == LlmOAuthProvider::Kimi {
        insert_kimi_identity(&mut headers, &kimi_device_id()?)?;
    } else if provider == LlmOAuthProvider::Xai {
        super::xai_headers::insert_identity(&mut headers)?;
    }
    Ok(headers)
}

pub fn request_headers_with_identity(
    provider: LlmOAuthProvider,
    purpose: RequestPurpose,
    user_id: Option<&str>,
) -> Result<HeaderMap, String> {
    let mut headers = request_headers_for(provider, purpose)?;
    if provider == LlmOAuthProvider::Xai {
        if let Some(user_id) = user_id {
            super::xai_headers::insert_user(&mut headers, user_id)?;
        }
    }
    Ok(headers)
}

pub fn request_headers_for(
    provider: LlmOAuthProvider,
    purpose: RequestPurpose,
) -> Result<HeaderMap, String> {
    if provider == LlmOAuthProvider::Kimi && purpose == RequestPurpose::AccountMetadata {
        let mut headers = HeaderMap::new();
        insert_static(&mut headers, "accept", "application/json")?;
        return Ok(headers);
    }
    request_headers(provider)
}

pub fn user_agent() -> String {
    brand::user_agent()
}

fn kimi_device_id() -> Result<String, String> {
    let path = paths::data_dir().join("oauth-providers/kimi-device-id");
    load_or_create_device_id(&path)
}

fn load_or_create_device_id(path: &Path) -> Result<String, String> {
    if let Ok(value) = read_device_id(path) {
        return Ok(value);
    }
    let value = uuid::Uuid::new_v4().to_string();
    private_store::atomic_write(path, value.as_bytes())?;
    Ok(value)
}

fn read_device_id(path: &Path) -> Result<String, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| unavailable())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(unavailable());
    }
    let file = std::fs::File::open(path).map_err(|_| unavailable())?;
    let mut value = String::new();
    file.take(MAX_DEVICE_ID_BYTES + 1)
        .read_to_string(&mut value)
        .map_err(|_| unavailable())?;
    if value.len() > MAX_DEVICE_ID_BYTES as usize {
        return Err(unavailable());
    }
    uuid::Uuid::parse_str(value.trim()).map_err(|_| unavailable())?;
    private_store::repair_path(path)?;
    Ok(value.trim().to_string())
}

fn device_name() -> String {
    safe_header(&sysinfo::System::host_name().unwrap_or_else(|| "desktop".to_string()))
}

fn os_version() -> String {
    safe_header(&sysinfo::System::kernel_version().unwrap_or_else(|| std::env::consts::OS.into()))
}

fn device_model() -> String {
    safe_header(&format!(
        "{} {} {}",
        std::env::consts::OS,
        sysinfo::System::long_os_version().unwrap_or_default(),
        std::env::consts::ARCH,
    ))
}

fn insert_kimi_identity(headers: &mut HeaderMap, device_id: &str) -> Result<(), String> {
    insert_static(headers, "x-msh-platform", KIMI_PLATFORM_HEADER)?;
    insert_static(headers, "x-msh-version", env!("CARGO_PKG_VERSION"))?;
    insert_static(headers, "x-msh-device-name", &device_name())?;
    insert_static(headers, "x-msh-device-model", &device_model())?;
    insert_static(headers, "x-msh-os-version", &os_version())?;
    insert_static(headers, "x-msh-device-id", device_id)
}

fn safe_header(value: &str) -> String {
    let filtered: String = value
        .chars()
        .filter(|character| character.is_ascii_graphic() || *character == ' ')
        .take(120)
        .collect();
    if filtered.is_empty() {
        "unknown".to_string()
    } else {
        filtered
    }
}

fn insert_static(headers: &mut HeaderMap, name: &'static str, value: &str) -> Result<(), String> {
    insert(headers, HeaderName::from_static(name), value)
}

fn insert(headers: &mut HeaderMap, name: HeaderName, value: &str) -> Result<(), String> {
    let value = HeaderValue::from_str(value).map_err(|_| unavailable())?;
    headers.insert(name, value);
    Ok(())
}

fn unavailable() -> String {
    "provider_configuration_invalid".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_identifies_beaver() {
        assert_eq!(
            user_agent(),
            format!("Beaver/{}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn platform_compatibility_value_is_isolated() {
        assert_eq!(KIMI_PLATFORM_HEADER, "kimi_code_cli");
    }

    #[test]
    fn kimi_identity_contains_all_observed_headers_and_real_user_agent() {
        let mut headers = HeaderMap::new();
        insert(&mut headers, USER_AGENT, &user_agent()).unwrap();
        let device_id = uuid::Uuid::new_v4().to_string();
        insert_kimi_identity(&mut headers, &device_id).unwrap();
        for name in [
            "x-msh-platform",
            "x-msh-version",
            "x-msh-device-name",
            "x-msh-device-model",
            "x-msh-os-version",
            "x-msh-device-id",
        ] {
            assert!(headers.contains_key(name));
        }
        assert_eq!(
            headers[USER_AGENT],
            format!("Beaver/{}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn kimi_usage_uses_only_the_official_accept_header() {
        let headers =
            request_headers_for(LlmOAuthProvider::Kimi, RequestPurpose::AccountMetadata).unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[reqwest::header::ACCEPT], "application/json");
        assert!(!headers.contains_key(USER_AGENT));
        assert!(!headers.contains_key("x-msh-platform"));
    }

    #[test]
    fn kimi_device_id_is_stable_and_private() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("oauth/device-id");
        let first = load_or_create_device_id(&path).unwrap();
        let second = load_or_create_device_id(&path).unwrap();
        assert_eq!(first, second);
        assert!(uuid::Uuid::parse_str(&first).is_ok());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
