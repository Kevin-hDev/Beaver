#[derive(Debug)]
pub enum RequestError {
    Fatal(String),
    RetryWithoutTools(String),
    RetryWithoutImages(String),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fatal(message)
            | Self::RetryWithoutTools(message)
            | Self::RetryWithoutImages(message) => f.write_str(message),
        }
    }
}
