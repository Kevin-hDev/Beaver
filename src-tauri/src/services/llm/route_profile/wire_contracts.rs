use super::types::*;
use crate::services::provider_usage::UsageApiFormat;

pub(super) const OPENAI_CHAT_WIRE: WireContract = WireContract {
    family: WireFamily::OpenAiChatCompletions,
    fragments: FragmentMode::DifferentialFragments,
    tool_results: ToolResultPlacement::ToolRole,
    images: ImageFormat::OpenAiNested,
    usage: UsageApiFormat::ChatCompletions,
};

pub(super) const MISTRAL_CHAT_WIRE: WireContract = WireContract {
    images: ImageFormat::MistralFlat,
    ..OPENAI_CHAT_WIRE
};

pub(super) const RESPONSES_WIRE: WireContract = WireContract {
    family: WireFamily::OpenAiResponses,
    fragments: FragmentMode::SemanticEvents,
    tool_results: ToolResultPlacement::ResponsesItem,
    images: ImageFormat::ResponsesInput,
    usage: UsageApiFormat::Responses,
};

pub(super) const OLLAMA_WIRE: WireContract = WireContract {
    family: WireFamily::OllamaNative,
    fragments: FragmentMode::DifferentialFragments,
    tool_results: ToolResultPlacement::OllamaNative,
    images: ImageFormat::OllamaNative,
    usage: UsageApiFormat::ChatCompletions,
};

pub(super) const ANTHROPIC_WIRE: WireContract = WireContract {
    family: WireFamily::AnthropicMessages,
    fragments: FragmentMode::SemanticEvents,
    tool_results: ToolResultPlacement::UserToolResultBlock,
    images: ImageFormat::AnthropicBlock,
    usage: UsageApiFormat::AnthropicMessages,
};
