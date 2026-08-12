pub mod agent_import;
pub mod agent_local;
pub mod api_keys;
pub mod app_log;
#[cfg(test)]
mod app_log_tests;
pub mod attachment_access;
#[cfg(test)]
mod attachment_access_tests;
pub mod autostart_migration;
pub mod background_command;
pub mod brand;
#[cfg(test)]
mod brand_tests;
pub mod browser;
pub mod codex_client;
pub mod codex_oauth;
pub mod compress;
pub mod config;
pub mod e2e_profile;
#[cfg(all(test, feature = "e2e"))]
mod e2e_profile_tests;
pub mod env_detect;
pub mod extensions;
pub mod favorite_models;
pub mod file_watcher;
pub mod forecast;
pub mod gateway;
pub mod git;
pub mod git_context;
pub mod gpu_detect;
pub mod gpu_vram;
pub mod link_preview;
pub mod llm;
pub mod llm_oauth;
pub mod mascot;
pub mod mcp_bridge;
pub mod mcp_oauth;
pub mod model_downloads;
pub mod model_downloads_store;
mod model_downloads_store_queue;
#[cfg(test)]
mod model_downloads_store_tests;
pub mod model_downloads_types;
pub mod oauth_completion;
pub mod oauth_providers;
pub mod oauth_work;
#[cfg(test)]
mod oauth_work_tests;
pub mod ollama_env;
pub mod ollama_kill;
#[cfg(test)]
mod ollama_kill_tests;
pub mod ollama_lifecycle;
pub mod ollama_port;
pub mod ollama_ps;
pub mod owned_process;
#[cfg(test)]
mod owned_process_tests;
pub mod paths;
pub mod personality_injection;
pub mod private_store;
pub mod process_identity;
pub mod process_tree;
pub mod provider_usage;
pub mod reasoning;
mod reasoning_effort;
mod reasoning_google;
#[cfg(test)]
mod reasoning_tests;
pub mod runtime_background;
#[cfg(test)]
mod runtime_background_tests;
pub mod scheduler;
pub mod search;
pub mod searxng;
pub mod secure_http;
pub mod security_cleanup;
mod shutdown_completion;
#[cfg(test)]
mod shutdown_completion_tests;
pub mod stream_utils;
pub mod system_executable;
pub mod terminal;
#[cfg(test)]
pub(crate) mod test_runtime;
pub mod token_counting;
pub mod update_handoff;
pub mod update_health;
pub mod vault;
pub mod work_registry;
#[cfg(test)]
mod work_registry_tests;
