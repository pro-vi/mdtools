use std::str::FromStr;

use crate::core_error::CoreError;

/// An opaque fingerprint for one structural target.
///
/// This is deliberately distinct from a whole-document revision. The wire
/// representation remains the existing lowercase 16-character hex string.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TargetEtag(String);

impl TargetEtag {
    pub fn for_bytes(bytes: &[u8]) -> Self {
        Self(content_etag(bytes))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for TargetEtag {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for TargetEtag {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}

/// Compatibility helper for the versioned CLI wire model.
pub fn content_etag(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}
