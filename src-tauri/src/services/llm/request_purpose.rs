use crate::services::provider_usage::UsageOrigin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestPurpose {
    ManualChat,
    ExternalChannel,
    Automation,
    AccountMetadata,
    Unknown,
}

impl RequestPurpose {
    pub async fn for_session(session_id: &str) -> Self {
        match crate::services::provider_usage::origin_for_session(session_id).await {
            Some(UsageOrigin::ManualChat) => Self::ManualChat,
            Some(UsageOrigin::ExternalChannel) => Self::ExternalChannel,
            Some(UsageOrigin::Automation) => Self::Automation,
            None => Self::Unknown,
        }
    }

    pub const fn allows_interactive_oauth(self) -> bool {
        matches!(self, Self::ManualChat | Self::AccountMetadata)
    }
}

#[cfg(test)]
mod tests {
    use super::RequestPurpose;

    #[test]
    fn only_manual_chat_and_account_metadata_allow_interactive_oauth() {
        assert!(RequestPurpose::ManualChat.allows_interactive_oauth());
        assert!(RequestPurpose::AccountMetadata.allows_interactive_oauth());
        assert!(!RequestPurpose::ExternalChannel.allows_interactive_oauth());
        assert!(!RequestPurpose::Automation.allows_interactive_oauth());
        assert!(!RequestPurpose::Unknown.allows_interactive_oauth());
    }
}
