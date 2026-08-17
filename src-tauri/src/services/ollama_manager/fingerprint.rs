#![allow(dead_code)]

use serde::de::{Error as DeError, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use subtle::ConstantTimeEq;

const MAX_VERSION_BYTES: usize = 64;
const SHA256_HEX_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FingerprintError {
    InvalidVersion,
    InvalidSha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OllamaVersion(String);

impl OllamaVersion {
    pub fn parse(raw: &str) -> Result<Self, FingerprintError> {
        if raw.is_empty()
            || raw.len() > MAX_VERSION_BYTES
            || raw.starts_with('v')
            || raw.chars().any(char::is_whitespace)
        {
            return Err(FingerprintError::InvalidVersion);
        }
        let parsed = semver::Version::parse(raw).map_err(|_| FingerprintError::InvalidVersion)?;
        if parsed.to_string() != raw {
            return Err(FingerprintError::InvalidVersion);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OllamaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for OllamaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OllamaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'de> Visitor<'de> for VersionVisitor {
            type Value = OllamaVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a canonical Ollama semantic version")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                OllamaVersion::parse(value).map_err(|_| E::custom("invalid Ollama version"))
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

#[derive(Clone, Debug)]
pub struct Sha256Digest {
    bytes: [u8; 32],
}

impl Sha256Digest {
    pub fn from_hex(raw: &str) -> Result<Self, FingerprintError> {
        if raw.len() != SHA256_HEX_BYTES || !raw.is_ascii() {
            return Err(FingerprintError::InvalidSha256);
        }
        let mut bytes = [0_u8; 32];
        for (index, pair) in raw.as_bytes().chunks_exact(2).enumerate() {
            bytes[index] = decode_hex_pair(pair).ok_or(FingerprintError::InvalidSha256)?;
        }
        Ok(Self { bytes })
    }

    pub fn to_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(SHA256_HEX_BYTES);
        for byte in self.bytes {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }

    pub fn constant_time_eq(&self, other: &Self) -> bool {
        bool::from(self.bytes.ct_eq(&other.bytes))
    }

    #[allow(dead_code)]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl PartialEq for Sha256Digest {
    fn eq(&self, other: &Self) -> bool {
        self.constant_time_eq(other)
    }
}

impl Eq for Sha256Digest {}

impl Serialize for Sha256Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ShaVisitor;

        impl<'de> Visitor<'de> for ShaVisitor {
            type Value = Sha256Digest;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("64 hexadecimal SHA-256 characters")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Sha256Digest::from_hex(value).map_err(|_| E::custom("invalid SHA-256 digest"))
            }
        }

        deserializer.deserialize_str(ShaVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleFingerprint {
    pub version: OllamaVersion,
    pub executable_sha256: Sha256Digest,
}

fn decode_hex_pair(pair: &[u8]) -> Option<u8> {
    Some((hex_value(pair.first().copied()?)? << 4) | hex_value(pair.get(1).copied()?)?)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
