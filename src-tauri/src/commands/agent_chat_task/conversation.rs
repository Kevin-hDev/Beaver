use crate::services::agent_local::conversation_admission::AdmittedTurn;
use crate::services::agent_local::conversation_history::{ProviderMessage, ProviderRole};
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

pub(crate) enum StreamConversation {
    Canonical {
        admitted: AdmittedTurn,
        system_prompt: Option<String>,
        subagent_owner: Option<(String, String)>,
    },
}

impl StreamConversation {
    pub(crate) fn canonical(admitted: AdmittedTurn) -> Self {
        Self::Canonical {
            admitted,
            system_prompt: None,
            subagent_owner: None,
        }
    }

    pub(crate) fn canonical_for_subagent(
        admitted: AdmittedTurn,
        system_prompt: String,
        run_id: String,
        execution_id: String,
    ) -> Self {
        Self::Canonical {
            admitted,
            system_prompt: Some(system_prompt),
            subagent_owner: Some((run_id, execution_id)),
        }
    }

    pub(crate) fn into_messages(self) -> Result<Vec<ChatMessage>, String> {
        match self {
            Self::Canonical {
                admitted,
                system_prompt,
                ..
            } => {
                let mut messages = admitted
                    .history
                    .messages
                    .into_iter()
                    .map(convert)
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(system_prompt) = system_prompt {
                    messages.insert(0, ChatMessage::system(system_prompt));
                }
                Ok(messages)
            }
        }
    }

    pub(crate) fn into_messages_and_journal(
        self,
        session_id: String,
        request_id: String,
    ) -> Result<
        (
            Vec<ChatMessage>,
            Option<crate::services::agent_local::conversation_journal::ConversationJournal>,
        ),
        String,
    > {
        match self {
            Self::Canonical {
                admitted,
                system_prompt,
                subagent_owner,
            } => {
                let is_compress_command = admitted.history.messages.last().is_some_and(|message| {
                    message.role == ProviderRole::User
                        && crate::services::compress::command::is_explicit_compression_command(
                            &message.content,
                        )
                });
                let journal = if is_compress_command {
                    None
                } else if let Some((run_id, execution_id)) = subagent_owner {
                    Some(crate::services::agent_local::conversation_journal::ConversationJournal::new_for_subagent(
                        session_id,
                        admitted.turn_id.clone(),
                        admitted.user_message_id.clone(),
                        admitted.assistant_message_id.clone(),
                        request_id,
                        run_id,
                        execution_id,
                    )?)
                } else {
                    Some(crate::services::agent_local::conversation_journal::ConversationJournal::new(
                        session_id,
                        admitted.turn_id.clone(),
                        admitted.user_message_id.clone(),
                        admitted.assistant_message_id.clone(),
                        request_id,
                    )?)
                };
                Ok((
                    Self::Canonical {
                        admitted,
                        system_prompt,
                        subagent_owner: None,
                    }
                    .into_messages()?,
                    journal,
                ))
            }
        }
    }
}

pub(crate) fn convert(message: ProviderMessage) -> Result<ChatMessage, String> {
    let images = (!message.images.is_empty()).then(|| {
        message
            .images
            .into_iter()
            .map(|image| image.base64)
            .collect()
    });
    let role = match message.role {
        ProviderRole::User => "user",
        ProviderRole::Assistant => "assistant",
        ProviderRole::Tool => "tool",
    };
    let tool_calls = message.tool_calls.map(|calls| {
        calls
            .into_iter()
            .map(|call| ToolCallOllama {
                id: Some(call.id),
                extra_content: call.extra_content,
                function: ToolCallFunction {
                    name: call.function.name,
                    arguments: call.function.arguments,
                },
            })
            .collect()
    });
    if message.tool_loop_reasoning.is_some() {
        return Err(generic_error());
    }
    Ok(ChatMessage {
        continuity_barrier_before: message.continuity_barrier_before,
        role: role.to_string(),
        content: message.content,
        images,
        tool_calls,
        tool_name: message.tool_name,
        tool_call_id: message.tool_call_id,
        display_thinking: message.display_thinking,
        continuation: message.continuation,
        tool_loop_reasoning: None,
    })
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}

#[cfg(test)]
#[path = "conversation_tests.rs"]
mod tests;
