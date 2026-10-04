//! This module owns one semantic global retention head.

use super::{
    LivenessGeneration, RetentionHeadError, RetentionManifestDigest, RetentionManifestLength,
};

/// Exact coordinate of the globally selected retention manifest.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionHead {
    generation: LivenessGeneration,
    manifest_length: RetentionManifestLength,
    manifest_digest: RetentionManifestDigest,
    predecessor: Option<RetentionManifestDigest>,
}

impl RetentionHead {
    /// Admits one semantic global retention-head coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionHeadError`] when initial or successor history is
    /// malformed.
    pub fn new(
        generation: LivenessGeneration,
        manifest_length: RetentionManifestLength,
        manifest_digest: RetentionManifestDigest,
        predecessor: Option<RetentionManifestDigest>,
    ) -> Result<Self, RetentionHeadError> {
        admit_predecessor(generation, predecessor)?;
        Ok(Self {
            generation,
            manifest_length,
            manifest_digest,
            predecessor,
        })
    }

    /// Returns the selected global liveness generation.
    pub const fn generation(self) -> LivenessGeneration {
        self.generation
    }

    /// Returns the exact selected manifest length.
    pub const fn manifest_length(self) -> RetentionManifestLength {
        self.manifest_length
    }

    /// Returns the exact selected manifest digest.
    pub const fn manifest_digest(self) -> RetentionManifestDigest {
        self.manifest_digest
    }

    /// Returns the preceding manifest digest, if this is a successor.
    pub const fn predecessor(self) -> Option<RetentionManifestDigest> {
        self.predecessor
    }
}

fn admit_predecessor(
    generation: LivenessGeneration,
    predecessor: Option<RetentionManifestDigest>,
) -> Result<(), RetentionHeadError> {
    if generation.get() == 1 {
        return predecessor.map_or(Ok(()), |observed| {
            Err(RetentionHeadError::InitialGenerationHasPredecessor { observed })
        });
    }
    if predecessor.is_some() {
        Ok(())
    } else {
        Err(RetentionHeadError::MissingPredecessor { generation })
    }
}
