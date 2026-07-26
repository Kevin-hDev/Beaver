use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

const PORT_RANGE_START: u16 = 12000;
const PORT_RANGE_END: u16 = 12099;
const DEFAULT_PORT: u16 = 12000;

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

pub fn clear_port() {
    ACTIVE_PORT.store(0, Ordering::Relaxed);
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

pub fn health_info(port: u16, auth_token: &str) -> Option<(u16, String, String)> {
    use std::io::{Read, Write};

    let addr = format!("127.0.0.1:{port}");
    let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().ok()?, Duration::from_secs(2))
    else {
        return None;
    };
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let req = format!(
        "GET /health HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\nX-CLGO-Forecast-Token: {auth_token}\r\n\r\n"
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return None;
    }
    let mut buf = [0u8; 512];
    let n = stream.read(&mut buf).unwrap_or(0);
    let response = String::from_utf8_lossy(&buf[..n]);
    let body = response.split("\r\n\r\n").nth(1)?;
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let model = json["model"].as_str()?.to_string();
    let family = json["family"].as_str().unwrap_or("").to_string();
    Some((port, model, family))
}
