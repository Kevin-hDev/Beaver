pub const MAX_PROFILES: usize = 20;
pub const MAX_PROFILE_NAME_CHARS: usize = 48;
pub const MAX_CUSTOM_PROMPT_CHARS: usize = 32_000;
pub const MAX_CATEGORY_ITEMS: u16 = 100;
pub const MAX_BUDGET_TOKENS: u32 = 1_000_000;
pub const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_RETRIES: u8 = 2;

pub const MAX_MODEL_FIELD_CHARS: usize = 256;

// Settings has no session payload to measure, so its preview uses one
// conservative backend-owned estimate until Task 16 can expose measurements.
pub const SETTINGS_SYSTEM_TOOLS_ESTIMATE: u32 = 12_000;
pub const SETTINGS_IMAGE_TOKEN_ESTIMATE: u32 = 1_024;
