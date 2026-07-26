pub const DISPLAY_NAME: &str = "Beaver";
pub const MCP_CLIENT_NAME: &str = "Beaver";
pub const USER_AGENT_PRODUCT: &str = "Beaver";
pub const GIT_AUTHOR_NAME_CONFIG: &str = "user.name=Beaver";
pub const GIT_AUTHOR_EMAIL_CONFIG: &str = "user.email=beaver@local";
pub const DIRECTORY_BASELINE_COMMIT_MESSAGE: &str = "Beaver directory baseline";

pub fn user_agent() -> String {
    format!("{USER_AGENT_PRODUCT}/{}", env!("CARGO_PKG_VERSION"))
}

pub fn directory_change_commit_message(id: &str) -> String {
    format!("Beaver temporary directory change\n\nBeaver-Subagent-Change: {id}")
}

pub fn subagent_change_commit_message(id: &str) -> String {
    format!("Beaver temporary subagent change\n\nBeaver-Subagent-Change: {id}")
}
