use crate::services::reasoning_continuity::contract::ContractId;
use crate::services::reasoning_continuity::limits::{checked_tool_calls, LimitError};
use crate::services::reasoning_continuity::tool_links::{self, ToolLink};

use super::ReasoningCapture;

impl ReasoningCapture {
    /// Ollama n'expose pas d'identifiant d'appel : le stream crée un UUID local
    /// qui doit aussi être lié à l'enveloppe opaque persistée.
    pub(crate) fn observe_native_tool_link(&mut self, provider_call_id: String, tool_name: String) {
        if self.partial {
            return;
        }
        let link = ToolLink {
            provider_call_id,
            tool_name,
        };
        let result = (|| {
            if self.contract_id != ContractId::OllamaNativeV1
                || self
                    .response_tool_links
                    .iter()
                    .any(|existing| existing.provider_call_id == link.provider_call_id)
            {
                return Err(LimitError::ProviderCallId);
            }
            checked_tool_calls(self.response_tool_links.len(), 1)?;
            tool_links::validate(std::slice::from_ref(&link))?;
            self.response_tool_links.push(link);
            Ok(())
        })();
        if result.is_err() {
            self.mark_partial();
        }
    }
}
