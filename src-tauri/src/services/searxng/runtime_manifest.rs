use serde::Deserialize;
use std::path::Path;
use subtle::ConstantTimeEq;

use super::runtime_error::RuntimeError;

pub(super) const MANIFEST_NAME: &str = ".runtime.json";
const MAX_MANIFEST_BYTES: u64 = 512;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RuntimeManifest {
    pub(super) python_major: u8,
    pub(super) python_minor: u8,
    requirements_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeManifestWire {
    schema_version: u8,
    implementation: String,
    major: u8,
    minor: u8,
    requirements_sha256: String,
}

impl RuntimeManifest {
    pub(super) fn read_from(wheelhouse: &Path) -> Result<Self, RuntimeError> {
        let path = wheelhouse.join(MANIFEST_NAME);
        let metadata =
            std::fs::symlink_metadata(path).map_err(|_| RuntimeError::ManifestInvalid)?;
        if !metadata.file_type().is_file() || metadata.len() > MAX_MANIFEST_BYTES {
            return Err(RuntimeError::ManifestInvalid);
        }
        let bytes = std::fs::read(wheelhouse.join(MANIFEST_NAME))
            .map_err(|_| RuntimeError::ManifestInvalid)?;
        Self::parse_bounded(&bytes)
    }

    pub(super) fn parse_bounded(bytes: &[u8]) -> Result<Self, RuntimeError> {
        if bytes.is_empty() || bytes.len() as u64 > MAX_MANIFEST_BYTES {
            return Err(RuntimeError::ManifestInvalid);
        }
        let wire: RuntimeManifestWire =
            serde_json::from_slice(bytes).map_err(|_| RuntimeError::ManifestInvalid)?;
        if wire.schema_version != 1
            || wire.implementation != "cpython"
            || wire.major != 3
            || !(10..=99).contains(&wire.minor)
            || !is_sha256(&wire.requirements_sha256)
        {
            return Err(RuntimeError::ManifestInvalid);
        }
        Ok(Self {
            python_major: wire.major,
            python_minor: wire.minor,
            requirements_sha256: wire.requirements_sha256,
        })
    }

    pub(super) fn matches_stamp(&self, stamp: &str) -> bool {
        is_sha256(stamp)
            && self
                .requirements_sha256
                .as_bytes()
                .ct_eq(stamp.as_bytes())
                .into()
    }

    #[cfg(test)]
    pub(super) fn for_test(python_major: u8, python_minor: u8) -> Self {
        Self {
            python_major,
            python_minor,
            requirements_sha256: "a".repeat(64),
        }
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
