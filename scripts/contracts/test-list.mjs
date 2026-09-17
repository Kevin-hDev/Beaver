// Exact Rust tests owned by contracts:check.
export const CONTRACT_TESTS = Object.freeze([
  "services::automations::contract_export::checked_in_automation_contract_matches_rust",
  "services::extensions::install_jobs::contract_tests::checked_in_typescript_matches_rust",
  "models::provider_contract_tests::checked_in_typescript_matches_the_rust_contract",
  "services::llm::model_reasoning_contract_tests::checked_in_typescript_matches_the_rust_reasoning_contract",
  "services::agent_local::types_diagnostics_contract_tests::checked_in_typescript_matches_the_rust_diagnostics_contract",
  "services::update_progress::tests::checked_in_typescript_matches_the_rust_update_progress_contract",
  "models::agent_session_contract_tests::checked_in_agent_session_types_match_rust",
  "models::compression_profile_contract_tests::checked_in_compression_profile_types_match_rust",
  "models::voice_contract_tests::checked_in_voice_types_match_rust",
  "services::model_downloads_contract_tests::checked_in_model_download_types_match_rust",
  "services::extensions::contract_artifact_tests::checked_in_typescript_matches_the_extension_contract",
  "services::extensions::contract_artifact_tests::checked_in_sdk_contract_matches_the_extension_contract",
  "services::extensions::contract_artifact_tests::checked_in_sdk_readme_tables_match_the_contract",
  "services::extensions::contract_artifact_tests::checked_in_private_document_tables_match_the_contract",
  "services::extensions::contract_artifact_tests::checked_in_ui_contract_artifacts_name_the_json_authority",
  "services::extensions::contract_artifact_tests::sdk_readme_has_one_bounded_ui_generated_section",
  "services::extensions::core_api_contract_tests::core_api_contract_is_single_authority",
  "services::agent_local::extension_discovery_contract_tests::discovery_contract_defines_the_r0_names_limits_and_host_imports",
  "services::agent_local::extension_discovery_contract_tests::discovery_contract_rejects_duplicate_or_copied_authority_keys",
  "services::agent_local::extension_discovery_contract_tests::discovery_bootstrap_and_generated_rust_are_bounded",
  "services::agent_local::extension_discovery_contract_tests::validation_proves_worst_case_json_budgets_without_truncation"
]);
