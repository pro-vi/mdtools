use std::str::FromStr;

use serde::Serialize;

use crate::core_error::CoreError;

/// An opaque fingerprint for one resolved target state.
///
/// This is deliberately distinct from a whole-document revision. The wire
/// representation remains the existing lowercase 16-character hex string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct TargetEtag(String);

/// A caller-supplied comparison token.
///
/// Unlike [`TargetEtag`], this may contain any string so the versioned CLI can
/// preserve its historical "malformed token means mismatch" behavior. It can
/// never be returned by a read operation as a valid target fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetEtagGuard(String);

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

impl TargetEtagGuard {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TargetEtagGuard {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<TargetEtag> for TargetEtagGuard {
    fn from(value: TargetEtag) -> Self {
        Self(value.into_string())
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
        if value.len() == 16
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value.to_string()))
        } else {
            Err(CoreError::InvalidTargetEtag(value.to_string()))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_etag_accepts_only_the_generated_wire_format() {
        let generated = TargetEtag::for_bytes(b"target");
        assert_eq!(
            generated.to_string().parse::<TargetEtag>().unwrap(),
            generated
        );
        for invalid in ["", "abc", "0123456789ABCDEf", "0123456789abcdeg"] {
            assert!(matches!(
                invalid.parse::<TargetEtag>(),
                Err(CoreError::InvalidTargetEtag(_))
            ));
        }
    }
}
