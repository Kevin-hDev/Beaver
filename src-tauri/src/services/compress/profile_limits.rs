pub const MAX_PROFILES: usize = 20;
// Lecture tolérante : laisse la normalisation remplacer les entrées invalides
// tout en bornant la collection intermédiaire issue du disque.
pub const MAX_PROFILE_READ_CANDIDATES: usize = MAX_PROFILES * 4;
pub const MAX_PROFILE_NAME_CHARS: usize = 48;
pub const MAX_CUSTOM_PROMPT_CHARS: usize = 32_000;

pub const MAX_MESSAGES: u8 = 8;
pub const MAX_TOOL_RESULTS: u16 = 50;
pub const MAX_FILES: u16 = 15;
pub const MAX_IMAGES: u16 = 16;
pub const MIN_SUMMARY_TOKENS: u32 = 1_000;
pub const MAX_SUMMARY_TOKENS: u32 = 8_000;

pub const SETTINGS_CONTEXT_TOKENS: u32 = 96_000;
pub const SETTINGS_SYSTEM_TOOLS_TOKENS: u32 = 12_000;
