use serde::{Deserialize, Serialize};

pub const MAX_MEMORY_CONTEXT_TOKENS: u32 = 3_000;
pub const DEFAULT_MEMORY_CONTEXT_TOKENS: u32 = 3_000;
pub const MAX_TOPICS_PER_SCOPE: usize = 256;
pub const MAX_TOPIC_BYTES: usize = 48 * 1024;
pub const MAX_SUMMARY_BYTES: usize = 16 * 1024;
pub const MAX_TAGS: usize = 8;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryMode {
    #[default]
    Disabled,
    Manual,
    Automatic,
}

impl MemoryMode {
    pub fn is_active(self) -> bool {
        self != Self::Disabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettings {
    #[serde(default)]
    pub mode: MemoryMode,
    #[serde(default = "default_context_tokens")]
    pub context_budget_tokens: u32,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            mode: MemoryMode::Disabled,
            context_budget_tokens: DEFAULT_MEMORY_CONTEXT_TOKENS,
        }
    }
}

impl MemorySettings {
    pub fn normalized(mut self) -> Self {
        self.context_budget_tokens = self
            .context_budget_tokens
            .clamp(256, MAX_MEMORY_CONTEXT_TOKENS);
        self
    }
}

fn default_context_tokens() -> u32 {
    DEFAULT_MEMORY_CONTEXT_TOKENS
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryTopic {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub memory_type: String,
    pub status: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub source: String,
    pub session_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryScopeOverview {
    pub id: String,
    pub label: String,
    pub topic_count: usize,
    pub total_bytes: u64,
    pub last_updated: Option<String>,
    pub topics: Vec<MemoryTopic>,
    pub topics_loaded: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryOverview {
    pub settings: MemorySettings,
    pub global: MemoryScopeOverview,
    pub active_project: Option<MemoryScopeOverview>,
    pub other_projects: Vec<MemoryScopeOverview>,
    pub legacy_detected: bool,
}
