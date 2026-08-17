#![allow(dead_code)]

use async_trait::async_trait;
use std::net::TcpListener;
use std::num::NonZeroU16;
use tokio::net::TcpStream;

use super::constants::{MAX_PROBE_PORT_ATTEMPTS, PROBE_CONNECT_TIMEOUT, PROBE_DEFAULT_PORT};
use super::error::OllamaErrorCode;
use super::types::OllamaEndpoint;

#[async_trait]
pub trait OllamaPortAllocator: Send + Sync {
    fn allocate_loopback(&self, excluded: &[u16]) -> Result<OllamaEndpoint, OllamaErrorCode>;
    async fn detect_external(&self) -> Result<Option<OllamaEndpoint>, OllamaErrorCode>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefaultOllamaPortAllocator {
    external_port: u16,
}

impl Default for DefaultOllamaPortAllocator {
    fn default() -> Self {
        Self {
            external_port: PROBE_DEFAULT_PORT,
        }
    }
}

impl DefaultOllamaPortAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub(crate) fn with_external_port(port: u16) -> Self {
        Self {
            external_port: port,
        }
    }
}

#[async_trait]
impl OllamaPortAllocator for DefaultOllamaPortAllocator {
    fn allocate_loopback(&self, excluded: &[u16]) -> Result<OllamaEndpoint, OllamaErrorCode> {
        for _ in 0..MAX_PROBE_PORT_ATTEMPTS {
            let listener = TcpListener::bind(("127.0.0.1", 0))
                .map_err(|_| OllamaErrorCode::OllamaValidationDeferred)?;
            let port = listener
                .local_addr()
                .ok()
                .and_then(|address| NonZeroU16::new(address.port()))
                .ok_or(OllamaErrorCode::OllamaValidationDeferred)?;
            if !excluded
                .iter()
                .take(MAX_PROBE_PORT_ATTEMPTS)
                .any(|excluded_port| *excluded_port == port.get())
            {
                return Ok(OllamaEndpoint::loopback(port));
            }
        }
        Err(OllamaErrorCode::OllamaValidationDeferred)
    }

    async fn detect_external(&self) -> Result<Option<OllamaEndpoint>, OllamaErrorCode> {
        let Some(port) = NonZeroU16::new(self.external_port) else {
            return Ok(None);
        };
        let endpoint = OllamaEndpoint::loopback(port);
        match tokio::time::timeout(
            PROBE_CONNECT_TIMEOUT,
            TcpStream::connect(("127.0.0.1", port.get())),
        )
        .await
        {
            Ok(Ok(_stream)) => Ok(Some(endpoint)),
            Ok(Err(error)) if error.kind() == std::io::ErrorKind::ConnectionRefused => Ok(None),
            Ok(Err(_)) => Err(OllamaErrorCode::OllamaValidationDeferred),
            // Un endpoint loopback qui ne répond pas n'est pas un démon externe
            // utilisable ; le sidecar possédé démarrera sur un autre port.
            Err(_) => Ok(None),
        }
    }
}
