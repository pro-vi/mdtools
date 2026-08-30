use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::core_error::CoreError;

/// An opaque fingerprint for one resolved target state.
///
/// This is deliberately distinct from a whole-document revision. The wire
/// representation is a full lowercase SHA-256 digest.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct TargetEtag(#[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))] String);

impl TargetEtag {
    pub fn for_bytes(bytes: &[u8]) -> Self {
        Self(format!("{:x}", Sha256::digest(bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
        if value.len() == 64
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

impl<'de> Deserialize<'de> for TargetEtag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_etag_accepts_only_the_generated_wire_format() {
        let generated = TargetEtag::for_bytes(b"target");
        assert_eq!(generated.as_str().len(), 64);
        assert_eq!(
            generated.to_string().parse::<TargetEtag>().unwrap(),
            generated
        );
        for invalid in [
            "",
            "abc",
            "0123456789abcdef",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeG",
        ] {
            assert!(matches!(
                invalid.parse::<TargetEtag>(),
                Err(CoreError::InvalidTargetEtag(_))
            ));
        }
    }

    #[test]
    fn target_etag_matches_the_sha256_known_answer() {
        assert_eq!(
            TargetEtag::for_bytes(b"abc").as_str(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
