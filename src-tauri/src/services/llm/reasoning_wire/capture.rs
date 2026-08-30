use crate::services::reasoning_continuity::contract::{ContractId, RouteId};
use crate::services::reasoning_continuity::envelope::ContinuationState;

pub(super) fn contract_for(route: RouteId) -> Option<ContractId> {
    crate::services::reasoning_continuity::registry::route_contract(route)
}

pub(super) fn empty_continuation(contract: ContractId) -> ContinuationState {
    match contract {
        ContractId::OllamaNativeV1 => ContinuationState::OllamaNative {
            thinking: String::new(),
        },
        ContractId::GeminiCompatV1 => ContinuationState::GeminiParts { parts: Vec::new() },
        ContractId::MistralChunksV1 => ContinuationState::MistralChunks { chunks: Vec::new() },
        ContractId::AnthropicMessagesV1 => {
            ContinuationState::AnthropicBlocks { blocks: Vec::new() }
        }
        ContractId::CerebrasChatV1 => ContinuationState::CerebrasReasoning {
            reasoning: String::new(),
        },
        ContractId::OpenRouterDetailsV1 => ContinuationState::OpenRouterDetails {
            details: Vec::new(),
        },
        ContractId::OpenAiResponsesV1
        | ContractId::XaiResponsesV1
        | ContractId::CodexResponsesV1 => ContinuationState::ResponsesLocal { items: Vec::new() },
        ContractId::DeepSeekChatV1
        | ContractId::KimiChatV1
        | ContractId::ZaiChatV1
        | ContractId::QwenChatV1 => ContinuationState::ChatReasoning {
            reasoning_content: String::new(),
        },
    }
}

pub(super) fn has_native_items(continuation: &ContinuationState) -> bool {
    matches!(
        continuation,
        ContinuationState::AnthropicBlocks { .. }
            | ContinuationState::GeminiParts { .. }
            | ContinuationState::MistralChunks { .. }
            | ContinuationState::OpenRouterDetails { .. }
            | ContinuationState::ResponsesLocal { .. }
    )
}
