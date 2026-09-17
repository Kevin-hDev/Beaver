use serde::Deserialize;

use super::session_limits::CURRENT_SESSION_SCHEMA_VERSION;

#[derive(Deserialize)]
struct VersionProbe {
    schema_version: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionVersion {
    Legacy(u16),
    Current,
    Future(u16),
}

impl SessionVersion {
    pub(super) const fn legacy_number(self) -> Option<u16> {
        match self {
            Self::Legacy(version) => Some(version),
            Self::Current | Self::Future(_) => None,
        }
    }
}

pub(super) fn version(bytes: &[u8]) -> Result<SessionVersion, String> {
    let probe: VersionProbe =
        serde_json::from_slice(bytes).map_err(|_| super::session_limits::invalid_session())?;
    match probe.schema_version {
        None | Some(1) => Ok(SessionVersion::Legacy(1)),
        Some(value) if (2..CURRENT_SESSION_SCHEMA_VERSION).contains(&value) => {
            Ok(SessionVersion::Legacy(value))
        }
        Some(CURRENT_SESSION_SCHEMA_VERSION) => Ok(SessionVersion::Current),
        Some(value) if value > CURRENT_SESSION_SCHEMA_VERSION => Ok(SessionVersion::Future(value)),
        Some(_) => Err(super::session_limits::invalid_session()),
    }
}
