pub mod chat_prompts;
pub(crate) mod chat_prompt_sections;
#[cfg(test)]
mod chat_prompts_behavior_tests;
#[cfg(test)]
pub mod chat_prompts_chat_tests;
#[cfg(test)]
pub mod chat_prompts_tests;
#[cfg(test)]
pub mod chat_prompts_web_status_tests;
pub mod prompt_chat_compact;
pub mod prompt_chat_detailed;
pub mod prompt_compact;
pub mod prompt_compact_style;
pub mod prompt_detailed;
pub mod prompt_detailed_sections;
pub mod prompt_external_content;
pub mod prompt_interactive;
pub mod prompt_objective;
pub mod prompt_plan;
pub mod prompt_priority;
#[cfg(test)]
mod prompt_tool_guidance_tests;
