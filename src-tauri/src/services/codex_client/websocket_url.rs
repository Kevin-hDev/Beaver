pub(super) const CODEX_WEBSOCKET_URL: &str = "wss://chatgpt.com/backend-api/codex/responses";

#[derive(Clone, Copy)]
pub(super) enum WebSocketUrlPolicy {
    CodexProduction,
    #[cfg(test)]
    LoopbackTest,
}

pub(super) fn allowed(url: &str, policy: WebSocketUrlPolicy) -> bool {
    match policy {
        WebSocketUrlPolicy::CodexProduction => url == CODEX_WEBSOCKET_URL,
        #[cfg(test)]
        WebSocketUrlPolicy::LoopbackTest => {
            let Ok(parsed) = url::Url::parse(url) else {
                return false;
            };
            let loopback = match parsed.host() {
                Some(url::Host::Ipv4(address)) => address.is_loopback(),
                Some(url::Host::Ipv6(address)) => address.is_loopback(),
                _ => false,
            };
            parsed.scheme() == "ws"
                && parsed.username().is_empty()
                && parsed.password().is_none()
                && parsed.query().is_none()
                && parsed.fragment().is_none()
                && loopback
        }
    }
}
