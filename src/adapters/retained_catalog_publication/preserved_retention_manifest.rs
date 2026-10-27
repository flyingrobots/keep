//! This module owns the retention coordinates a catalog publication preserved.

use crate::{LivenessGeneration, RetentionManifestDigest};

/// Exact retention manifest whose every root closure a candidate preserved.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreservedRetentionManifest {
    generation: LivenessGeneration,
    digest: RetentionManifestDigest,
    root_count: u32,
}

impl PreservedRetentionManifest {
    /// Returns the preserved global liveness generation.
    pub const fn generation(self) -> LivenessGeneration {
        self.generation
    }

    /// Returns the preserved canonical manifest digest.
    pub const fn digest(self) -> RetentionManifestDigest {
        self.digest
    }

    /// Returns how many manifest roots were verified against the candidate.
    #[must_use]
    pub const fn root_count(self) -> u32 {
        self.root_count
    }

    pub(super) const fn new(
        generation: LivenessGeneration,
        digest: RetentionManifestDigest,
        root_count: u32,
    ) -> Self {
        Self {
            generation,
            digest,
            root_count,
        }
    }
}
