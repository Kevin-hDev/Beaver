use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::limits::{validate_provider_call_id, validate_tool_name, LimitError, MAX_TOOL_CALLS};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolLink {
    pub provider_call_id: String,
    pub tool_name: String,
}

pub fn validate(links: &[ToolLink]) -> Result<(), LimitError> {
    if links.len() > MAX_TOOL_CALLS {
        return Err(LimitError::ToolCalls);
    }
    let mut ids = HashSet::with_capacity(links.len());
    for link in links {
        validate_provider_call_id(&link.provider_call_id)?;
        validate_tool_name(&link.tool_name)?;
        if !ids.insert(link.provider_call_id.as_str()) {
            return Err(LimitError::ProviderCallId);
        }
    }
    Ok(())
}
