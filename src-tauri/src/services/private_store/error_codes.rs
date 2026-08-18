pub(crate) const PRIVATE_STORE_UNAVAILABLE: &str = "private-store-unavailable";
pub(crate) const PROJECT_STORE_UNAVAILABLE: &str = "project-store-unavailable";

pub(crate) const OLLAMA_CUSTOM_MISSING: &str = "ollama-custom-store-missing";
pub(crate) const OLLAMA_CUSTOM_UNAVAILABLE: &str = "ollama-custom-store-unavailable";
pub(crate) const OLLAMA_CUSTOM_WRITE: &str = "ollama-custom-store-write";

pub(crate) const OLLAMA_NATIVE_PROMPT_MISSING: &str = "ollama-native-prompt-store-missing";
pub(crate) const OLLAMA_NATIVE_PROMPT_UNAVAILABLE: &str = "ollama-native-prompt-store-unavailable";
pub(crate) const OLLAMA_NATIVE_PROMPT_WRITE: &str = "ollama-native-prompt-store-write";

pub(crate) const SYSTEM_PROMPT_MISSING: &str = "system-prompt-store-missing";
pub(crate) const SYSTEM_PROMPT_UNAVAILABLE: &str = "system-prompt-store-unavailable";
pub(crate) const SYSTEM_PROMPT_WRITE: &str = "system-prompt-store-write";

#[cfg(test)]
pub(crate) const LOCAL_STORE_CODES: [&str; 11] = [
    PRIVATE_STORE_UNAVAILABLE,
    PROJECT_STORE_UNAVAILABLE,
    OLLAMA_CUSTOM_MISSING,
    OLLAMA_CUSTOM_UNAVAILABLE,
    OLLAMA_CUSTOM_WRITE,
    OLLAMA_NATIVE_PROMPT_MISSING,
    OLLAMA_NATIVE_PROMPT_UNAVAILABLE,
    OLLAMA_NATIVE_PROMPT_WRITE,
    SYSTEM_PROMPT_MISSING,
    SYSTEM_PROMPT_UNAVAILABLE,
    SYSTEM_PROMPT_WRITE,
];
