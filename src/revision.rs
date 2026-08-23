use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::core_error::CoreError;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct DocumentRevision(String);

impl DocumentRevision {
    pub fn for_source(source: &str) -> Self {
        Self(format!("{:x}", Sha256::digest(source.as_bytes())))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
