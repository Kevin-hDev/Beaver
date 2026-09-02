use super::types::{ExtensionKind, ExtensionRecord};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum HostIdentity {
    Official,
    ThirdParty(String),
}

impl HostIdentity {
    pub(super) fn from_record(record: &ExtensionRecord) -> Result<Self, String> {
        match record.kind {
            ExtensionKind::Builtin => Ok(Self::Official),
            ExtensionKind::Local => Ok(Self::ThirdParty(record.manifest.id.clone())),
            ExtensionKind::External => Err(super::error_codes::HOST_UNAVAILABLE.to_string()),
        }
    }
}
