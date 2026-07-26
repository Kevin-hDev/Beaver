use std::net::TcpListener;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

const PORT_RANGE_START: u16 = 11500;
const PORT_RANGE_END: u16 = 11599;
const DEFAULT_PORT: u16 = 11434;

static ACTIVE_PORT: AtomicU16 = AtomicU16::new(0);

pub fn get_port() -> u16 {
    let port = ACTIVE_PORT.load(Ordering::Relaxed);
    if port == 0 {
        DEFAULT_PORT
    } else {
        port
    }
}

pub fn set_port(port: u16) {
    ACTIVE_PORT.store(port, Ordering::Relaxed);
}

pub fn base_url() -> String {
    format!("http://127.0.0.1:{}", get_port())
}

pub fn find_free_port() -> u16 {
    for port in PORT_RANGE_START..=PORT_RANGE_END {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    DEFAULT_PORT
}

pub fn is_port_open(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().unwrap(),
        Duration::from_millis(500),
    )
    .is_ok()
}

pub fn detect_existing_instance(port: u16) -> bool {
    if !is_port_open(port) {
        return false;
    }
    verify_ollama_api(port)
}

fn verify_ollama_api(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{port}/api/version");
    let handle = std::thread::Builder::new()
        .name("ollama-check".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            rt.block_on(async {
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(2))
                    .build()
                    .ok()?;
                let resp = client.get(&url).send().await.ok()?;
                if !resp.status().is_success() {
                    return None;
                }
                let json: serde_json::Value = resp.json().await.ok()?;
                json["version"].as_str().map(|_| true)
            })
        });
    match handle {
        Ok(h) => h.join().ok().flatten().unwrap_or(false),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_free_port_returns_valid_range() {
        let port = find_free_port();
        assert!(
            (PORT_RANGE_START..=PORT_RANGE_END).contains(&port) || port == DEFAULT_PORT,
            "port {port} hors range"
        );
    }

    #[test]
    fn set_and_get_port_roundtrip() {
        set_port(11550);
        assert_eq!(get_port(), 11550);
        set_port(0);
    }

    #[test]
    fn base_url_uses_active_port() {
        set_port(11555);
        assert_eq!(base_url(), "http://127.0.0.1:11555");
        set_port(0);
    }

    #[test]
    fn is_port_open_on_unused_port() {
        assert!(!is_port_open(59999));
    }
}
