use crate::services::reasoning_continuity::contract::ContractId;
use crate::services::reasoning_continuity::limits::{checked_tool_calls, LimitError};
use crate::services::reasoning_continuity::tool_links::{self, ToolLink};

use super::ReasoningCapture;

impl ReasoningCapture {
    /// Lie l'enveloppe au message assistant effectivement persisté. Les noms
    /// wire peuvent être des alias provider ; seuls les noms canoniques du
    /// résultat final sont durables et admissibles au redémarrage.
    pub(crate) fn observe_persisted_tool_links(
        &mut self,
        tool_calls: &[(String, serde_json::Value)],
        tool_call_ids: &[String],
    ) {
        if self.partial {
            return;
        }
        if tool_calls.len() != tool_call_ids.len() {
            self.mark_partial();
            return;
        }
        for ((tool_name, _), provider_call_id) in tool_calls.iter().zip(tool_call_ids) {
            let link = ToolLink {
                provider_call_id: provider_call_id.clone(),
                tool_name: tool_name.clone(),
            };
            let result = (|| {
                if self
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
                return;
            }
        }
    }

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
