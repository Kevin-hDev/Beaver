use super::types::*;
use crate::services::provider_usage::UsageApiFormat;

pub(super) const OPENAI_CHAT_WIRE: WireContract = WireContract {
    family: WireFamily::OpenAiChatCompletions,
    fragments: FragmentMode::DifferentialFragments,
    tool_results: ToolResultPlacement::ToolRole,
    images: ImageFormat::OpenAiNested,
    usage: UsageApiFormat::ChatCompletions,
    tool_result_media: ToolResultMedia::FollowUpUserMessage,
};

pub(super) const MISTRAL_CHAT_WIRE: WireContract = WireContract {
    images: ImageFormat::MistralFlat,
    tool_result_media: ToolResultMedia::FollowUpUserMessage,
    ..OPENAI_CHAT_WIRE
};

pub(super) const RESPONSES_WIRE: WireContract = WireContract {
    family: WireFamily::OpenAiResponses,
    fragments: FragmentMode::SemanticEvents,
    tool_results: ToolResultPlacement::ResponsesItem,
    images: ImageFormat::ResponsesInput,
    usage: UsageApiFormat::Responses,
    tool_result_media: ToolResultMedia::Inline,
};

// Ces routes partagent le fil Responses mais n'ont pas de preuve multimodale P6.
pub(super) const RESPONSES_TEXT_ONLY_WIRE: WireContract = WireContract {
    tool_result_media: ToolResultMedia::TextOnly,
    ..RESPONSES_WIRE
};

// Some xAI subscription catalog entries still use Chat Completions. They keep
// their own explicit text-only contract instead of borrowing Responses shape.
pub(super) const XAI_OAUTH_CHAT_TEXT_ONLY_WIRE: WireContract = WireContract {
    tool_result_media: ToolResultMedia::TextOnly,
    ..OPENAI_CHAT_WIRE
};

pub(super) const OLLAMA_WIRE: WireContract = WireContract {
    family: WireFamily::OllamaNative,
    fragments: FragmentMode::DifferentialFragments,
    tool_results: ToolResultPlacement::OllamaNative,
    images: ImageFormat::OllamaNative,
    usage: UsageApiFormat::ChatCompletions,
    tool_result_media: ToolResultMedia::FollowUpUserMessage,
};

pub(super) const ANTHROPIC_WIRE: WireContract = WireContract {
    family: WireFamily::AnthropicMessages,
    fragments: FragmentMode::SemanticEvents,
    tool_results: ToolResultPlacement::UserToolResultBlock,
    images: ImageFormat::AnthropicBlock,
    usage: UsageApiFormat::AnthropicMessages,
    tool_result_media: ToolResultMedia::Inline,
};
