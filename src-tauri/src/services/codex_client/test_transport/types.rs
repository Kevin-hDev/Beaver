#[derive(Debug, Clone, Copy)]
pub(crate) enum HttpReply {
    Unauthorized,
    Success,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum WebSocketReply {
    Success,
    ServiceTierRejected,
    Unavailable,
}

#[derive(Debug, Clone)]
pub(crate) struct HttpCapture {
    pub request: RequestProjection,
    pub routing_hint: Option<String>,
    pub authorization_valid: bool,
    pub account_header_present: bool,
    pub originator_valid: bool,
    pub user_agent_present: bool,
    pub response_path_valid: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct WebSocketCapture {
    pub request: RequestProjection,
    pub routing_hint: Option<String>,
    pub authorization_valid: bool,
    pub account_header_present: bool,
    pub originator_valid: bool,
    pub user_agent_present: bool,
    pub beta_header_valid: bool,
    pub session_headers_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestProjection {
    pub model: String,
    pub service_tier: Option<String>,
    pub envelope_type: Option<String>,
    pub input_count: usize,
    pub tool_count: usize,
    pub forbidden_field_present: bool,
    pub body_bytes: usize,
}
