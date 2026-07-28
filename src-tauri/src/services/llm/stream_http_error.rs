#[derive(Debug)]
pub enum RequestError {
    Fatal(String),
    PayloadTooLarge,
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal(message) => f.write_str(message),
            Self::PayloadTooLarge => f.write_str("provider_payload_too_large"),
        }
    }
}
