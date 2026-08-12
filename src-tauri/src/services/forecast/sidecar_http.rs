use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use zeroize::Zeroizing;

const PORT_RANGE_START: u16 = 12000;
const PORT_RANGE_END: u16 = 12099;
const DEFAULT_PORT: u16 = 12000;
const HEALTH_IO_PHASE_TIMEOUT: Duration = Duration::from_secs(2);
// Connect and read are sequential phases, so callers must allow their sum.
pub(super) const HEALTH_PROBE_BUDGET: Duration = Duration::from_secs(4);

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

pub fn find_free_port() -> u16 {
    for port in PORT_RANGE_START..=PORT_RANGE_END {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    DEFAULT_PORT
}

pub fn health_info(port: u16, auth_token: &Zeroizing<String>) -> Option<(u16, String, String)> {
    use std::io::{Read, Write};

    let addr = format!("127.0.0.1:{port}");
    let Ok(mut stream) = TcpStream::connect_timeout(&addr.parse().ok()?, HEALTH_IO_PHASE_TIMEOUT)
    else {
        return None;
    };
    stream.set_read_timeout(Some(HEALTH_IO_PHASE_TIMEOUT)).ok();
    let request_prefix =
        format!("GET /health HTTP/1.0\r\nHost: 127.0.0.1:{port}\r\nX-CLGO-Forecast-Token: ");
    if stream.write_all(request_prefix.as_bytes()).is_err()
        || stream.write_all(auth_token.as_bytes()).is_err()
        || stream.write_all(b"\r\n\r\n").is_err()
    {
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

#[cfg(test)]
mod tests {
    #[test]
    fn health_request_accepts_only_zeroizing_token_ownership() {
        let function: fn(u16, &zeroize::Zeroizing<String>) -> Option<(u16, String, String)> =
            super::health_info;
        let _ = function;
    }
}
