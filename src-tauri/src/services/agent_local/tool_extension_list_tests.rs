use super::*;

#[test]
fn listing_unavailable_uses_the_generated_contract_code() {
    let result = unavailable_result();

    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some(crate::services::extensions::error_codes::LISTING_UNAVAILABLE)
    );
}

#[test]
fn listing_is_constructed_without_a_session_mutation_capability() {
    let result = super::execute_with(|| Ok("[]".to_string()));

    assert!(!result.is_error);
    assert_eq!(result.content, "[]");
}
