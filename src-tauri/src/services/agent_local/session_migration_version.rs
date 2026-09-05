use serde::Deserialize;

#[derive(Deserialize)]
struct VersionProbe {
    schema_version: Option<u16>,
}

pub(super) enum WireVersion {
    V1,
    V2,
    V3,
    V4,
    V5,
    Future(u16),
}

pub(super) fn version(bytes: &[u8]) -> Result<WireVersion, String> {
    let probe: VersionProbe = serde_json::from_slice(bytes).map_err(|_| super::session_limits::invalid_session())?;
    match probe.schema_version {
        None | Some(1) => Ok(WireVersion::V1),
        Some(2) => Ok(WireVersion::V2),
        Some(3) => Ok(WireVersion::V3),
        Some(4) => Ok(WireVersion::V4),
        Some(super::session_migration_v5::SCHEMA_VERSION) => Ok(WireVersion::V5),
        Some(value) if value > super::session_limits::CURRENT_SESSION_SCHEMA_VERSION => {
            Ok(WireVersion::Future(value))
        }
        Some(_) => Err(super::session_limits::invalid_session()),
    }
}
