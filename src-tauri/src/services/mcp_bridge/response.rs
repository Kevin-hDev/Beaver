use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsonRpcResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
}

pub async fn parse(resp: reqwest::Response) -> Result<JsonRpcResponse, String> {
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let bytes = crate::services::secure_http::read_bounded(
        resp,
        crate::services::secure_http::MCP_BODY_LIMIT,
    )
    .await
    .map_err(|_| "réponse MCP invalide".to_string())?;

    let body_text = std::str::from_utf8(&bytes).map_err(|_| "réponse non UTF-8")?;

    if content_type.contains("text/event-stream") {
        return parse_sse(body_text);
    }

    serde_json::from_str(body_text).map_err(|_| "réponse JSON invalide".to_string())
}

fn parse_sse(sse_text: &str) -> Result<JsonRpcResponse, String> {
    for line in sse_text.lines() {
        let data = match line.strip_prefix("data: ") {
            Some(d) if !d.is_empty() => d.trim(),
            _ => continue,
        };
        if let Ok(parsed) = serde_json::from_str::<JsonRpcResponse>(data) {
            return Ok(parsed);
        }
    }
    Err("aucune réponse JSON-RPC dans le flux SSE".to_string())
}
