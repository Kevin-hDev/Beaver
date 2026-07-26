use rand::{rngs::OsRng, RngCore};
use std::net::TcpListener;
use std::path::PathBuf;

const TEMPLATE: &str = include_str!("../../../resources/searxng-sidecar/settings.template.yml");

pub fn find_free_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|_| "SearXNG: port local indisponible".to_string())?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|_| "SearXNG: port local indisponible".to_string())
}

pub fn write_settings(port: u16) -> Result<PathBuf, String> {
    let path = super::paths::settings_path();
    let secret = generate_secret();
    let body = render_settings(port, &secret);
    let dir = path
        .parent()
        .ok_or_else(|| "SearXNG: dossier config invalide".to_string())?;
    std::fs::create_dir_all(dir).map_err(|_| "SearXNG: config impossible".to_string())?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, body).map_err(|_| "SearXNG: config impossible".to_string())?;
    std::fs::rename(&tmp, &path).map_err(|_| "SearXNG: config impossible".to_string())?;
    Ok(path)
}

pub(crate) fn render_settings(port: u16, secret: &str) -> String {
    TEMPLATE
        .replace("{{PORT}}", &port.to_string())
        .replace("{{SECRET}}", secret)
}

fn generate_secret() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_settings_keeps_json_only_and_local_bind() {
        let rendered = render_settings(45123, "not-secret-in-test");
        assert!(rendered.contains("port: 45123"));
        assert!(rendered.contains("bind_address: \"127.0.0.1\""));
        assert!(rendered.contains("- json"));
        assert!(!rendered.contains("{{SECRET}}"));
    }
}
