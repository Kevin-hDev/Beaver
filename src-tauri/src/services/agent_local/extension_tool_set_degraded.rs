use super::ExtensionToolSet;
use crate::services::agent_local::{
    extension_tool_set_apply, stream_diagnostics_support, stream_events::AgentEventEmitter,
    tool_definitions, tool_extension_resource, types_ollama::StreamEvent,
};
use crate::services::extensions;
use serde_json::Value;

const DEGRADED_PHASE: &str = "extensions_unavailable";
const DEGRADED_SUMMARY: &str =
    "Extensions indisponibles ; seuls les outils natifs autorisés sont conservés.";

impl ExtensionToolSet {
    pub(super) fn degraded(
        tools: Vec<Value>,
        limit: usize,
        code: &'static str,
        session_id: &str,
    ) -> Result<Self, String> {
        // Only the canonical native catalog may supply schemas after a failure.
        // Intersect with the admitted names: never broaden a subagent or optional-tool policy.
        let lease = super::native_only::NativeOnlyLease::acquire(session_id)?;
        let native = tool_definitions::native_tool_definitions()
            .into_iter()
            .filter(|native| {
                let Some(name) = extension_tool_set_apply::definition_name(native) else {
                    return false;
                };
                !matches!(
                    name,
                    extensions::LIST_EXTENSIONS_TOOL_NAME
                        | extensions::INSPECT_EXTENSIONS_TOOL_NAME
                        | tool_extension_resource::NAME
                ) && tools
                    .iter()
                    .any(|tool| extension_tool_set_apply::definition_name(tool) == Some(name))
            })
            .collect();
        let capped = extension_tool_set_apply::cap_definitions(native, limit);
        let mut result = Self::passthrough(capped.tools);
        result.provider_tool_limit = limit;
        result.degradation = Some(code);
        result._native_only = Some(lease);
        result.omitted_tool_names = capped.omitted_tool_names;
        result.additional_omitted_tools = capped.additional_omitted_tools;
        Ok(result)
    }

    pub async fn report_prepared(
        &self,
        emitter: &AgentEventEmitter,
        session_id: &str,
        request_id: &str,
    ) -> Result<(), String> {
        if let Some(code) = self.degradation {
            log::warn!("[extensions] conversation_degraded code={code}");
            // This writes the conversation journal itself, not a separate log.
            // Refuse to execute tools if their conversation can no longer be saved.
            stream_diagnostics_support::update_run(session_id, request_id, |_, run| {
                run.severity = "warning".to_string();
                stream_diagnostics_support::push_event(
                    run,
                    DEGRADED_PHASE,
                    DEGRADED_SUMMARY,
                    None,
                    Some(code),
                );
            })
            .await?;
            send_notice(code, |event| emitter.send(event));
        }
        super::record_selection(self, session_id, request_id, "extension_tools_selected").await;
        Ok(())
    }
}

// Notice delivery is ancillary; the durable warning above remains available.
fn send_notice(code: &str, send: impl FnOnce(StreamEvent) -> Result<(), String>) {
    if send(StreamEvent::Notice {
        message_key: format!("extensions.errors.codes.{code}"),
    })
    .is_err()
    {
        log::warn!("[extensions] degradation_notice_delivery_failed");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn failed_notice_delivery_is_nonfatal() {
        let mut attempted = false;
        super::send_notice(
            crate::services::extensions::error_codes::STATE_UNAVAILABLE,
            |event| {
                attempted = true;
                assert!(matches!(event, super::StreamEvent::Notice { .. }));
                Err("delivery failed".to_string())
            },
        );
        assert!(attempted);
    }
}
