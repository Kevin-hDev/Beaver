use zeroize::Zeroizing;

pub struct StdioTransport {
    pub connector_id: String,
    pub install_command: String,
    pub env_key_names: Vec<String>,
    pub transient_env: Option<Vec<(String, Zeroizing<String>)>>,
    #[cfg(test)]
    pub(super) test_init_delay_ms: u64,
}
