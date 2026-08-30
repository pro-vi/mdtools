use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::core_error::CoreError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct DocumentRevision(
    #[schemars(length(equal = 64), regex(pattern = "^[0-9a-f]{64}$"))] String,
);

impl DocumentRevision {
    pub fn for_source(source: &str) -> Self {
        Self(format!("{:x}", Sha256::digest(source.as_bytes())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for DocumentRevision {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value.to_string()))
        } else {
            Err(CoreError::InvalidDocumentRevision(value.to_string()))
        }
    }
}

impl<'de> Deserialize<'de> for DocumentRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for DocumentRevision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub fn verify_source_revision(source: &str, expected: &DocumentRevision) -> Result<(), CoreError> {
    let actual = DocumentRevision::for_source(source);
    if &actual == expected {
        Ok(())
    } else {
        Err(CoreError::DocumentRevisionMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentRevision;

    #[test]
    fn document_revision_matches_the_sha256_known_answer() {
        assert_eq!(
            DocumentRevision::for_source("abc").as_str(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
