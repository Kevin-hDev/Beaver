pub mod callback;
mod callback_server;
pub mod jwt;
pub mod login;
pub mod pkce;
pub mod store;
pub mod token;
mod token_response;

pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub(crate) const ORIGINATOR: &str = "codex_cli_rs";
