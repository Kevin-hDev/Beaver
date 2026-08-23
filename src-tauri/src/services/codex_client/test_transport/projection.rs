use super::RequestProjection;

#[path = "projection_bounds_tests.rs"]
mod bounds_tests;
#[path = "projection_lex.rs"]
mod lex;
#[path = "projection_scan.rs"]
mod scan;
#[path = "projection_state.rs"]
mod state;
#[path = "projection_string.rs"]
mod string;
#[path = "projection_syntax_tests.rs"]
mod syntax_tests;
#[path = "projection_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug)]
struct ScanError;

pub(super) fn parse(body_bytes: &[u8]) -> Result<RequestProjection, String> {
    if body_bytes.len() > crate::services::secure_http::LLM_BODY_LIMIT {
        return Err(invalid());
    }
    // Une passe brute évite toute matérialisation Serde des valeurs ignorées sensibles.
    let scanned = scan::parse(body_bytes).map_err(|_| invalid())?;
    Ok(RequestProjection {
        model: scanned.model,
        service_tier: scanned.service_tier,
        envelope_type: scanned.envelope_type,
        input_count: scanned.input_count,
        tool_count: scanned.tool_count,
        forbidden_field_present: scanned.forbidden_field_present,
        body_bytes: body_bytes.len(),
    })
}

fn invalid() -> String {
    "provider_configuration_invalid".to_string()
}
