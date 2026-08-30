pub mod canonical_context;
pub mod checkpoint_attachments;
pub mod checkpoint_candidate;
mod checkpoint_candidate_budget;
mod checkpoint_candidate_images;
mod checkpoint_candidate_runtime;
pub mod checkpoint_document;
pub mod checkpoint_files;
pub mod checkpoint_live_state;
pub mod checkpoint_messages;
mod checkpoint_metadata;
pub mod checkpoint_reasoning;
mod checkpoint_reference_collect;
pub mod checkpoint_references;
mod checkpoint_retained_messages;
mod checkpoint_section_writer;
pub mod checkpoint_selection;
pub mod checkpoint_subagents;
mod checkpoint_text_attachments;
pub mod checkpoint_tools;
pub mod checkpoint_transaction;
pub mod checkpoint_units;
pub mod compression_redaction;
pub mod context_capsules_disk;
mod context_capsules_disk_collect;
pub mod context_resolve;
pub mod engine;
pub mod metrics;
mod metrics_facts;
mod metrics_projection;
pub mod orchestrator;
mod orchestrator_candidate;
mod orchestrator_metrics;
mod orchestrator_sections;
mod orchestrator_summary;
mod orchestrator_support;
pub mod profile_budget;
pub mod profile_defaults;
pub mod profile_limits;
mod profile_normalization;
pub mod profile_resolve;
pub mod profile_store;
pub mod profile_store_document;
mod profile_store_migration;
pub mod profile_types;
pub mod profile_validation;
pub mod prompt;
pub mod realtime_budget;
pub mod session_capabilities;
pub mod snapshot;
pub mod state;
#[cfg(test)]
pub(crate) mod state_recent;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
pub mod summary_budget;
pub mod summary_contract;
pub mod summary_request;
pub mod summary_retry;
pub mod timeouts;
pub mod token_estimate;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod end_to_end_tests;

#[cfg(test)]
mod metrics_tests;

#[cfg(test)]
mod cache_stability_tests;

#[cfg(test)]
mod profile_budget_tests;

#[cfg(test)]
mod profile_defaults_tests;

#[cfg(test)]
mod profile_resolve_tests;

#[cfg(test)]
mod profile_store_tests;

#[cfg(test)]
mod profile_validation_tests;

#[cfg(test)]
mod session_capabilities_tests;

#[cfg(test)]
mod snapshot_tests;

#[cfg(test)]
mod canonical_context_tests;

#[cfg(test)]
mod checkpoint_messages_tests;

#[cfg(test)]
mod checkpoint_sources_tests;

#[cfg(test)]
mod checkpoint_reasoning_tests;

#[cfg(test)]
mod checkpoint_tools_tests;

#[cfg(test)]
mod checkpoint_transaction_tests;

#[cfg(test)]
mod orchestrator_tests;

#[cfg(test)]
mod checkpoint_candidate_runtime_tests;

#[cfg(test)]
mod compression_redaction_tests;

#[cfg(test)]
mod summary_contract_tests;

#[cfg(test)]
mod summary_request_tests;

#[cfg(test)]
mod timeouts_tests;
