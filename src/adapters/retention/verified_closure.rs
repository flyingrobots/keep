//! This module owns successful retention-closure verification evidence.

use crate::{
    CatalogDigest, CatalogGeneration, RegisteredRetentionProfile, RetentionClosureDigest,
    RetentionClosureUsage,
};

/// Exact coordinates and accounting for one completely verified closure.
#[must_use = "verified closure evidence binds the transition's physical claim"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedRetentionClosure {
    profile: RegisteredRetentionProfile,
    catalog_generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    usage: RetentionClosureUsage,
    digest: RetentionClosureDigest,
}

impl VerifiedRetentionClosure {
    /// Returns the exact registered retention-realization profile.
    pub const fn profile(self) -> RegisteredRetentionProfile {
        self.profile
    }

    /// Returns the pinned catalog generation used for verification.
    pub const fn catalog_generation(self) -> CatalogGeneration {
        self.catalog_generation
    }

    /// Returns the pinned catalog digest used for verification.
    pub const fn catalog_digest(self) -> CatalogDigest {
        self.catalog_digest
    }

    /// Returns the exact successful resource accounting.
    pub const fn usage(self) -> RetentionClosureUsage {
        self.usage
    }

    /// Returns the canonical closure transcript digest.
    pub const fn digest(self) -> RetentionClosureDigest {
        self.digest
    }

    pub(super) const fn new(
        profile: RegisteredRetentionProfile,
        catalog_generation: CatalogGeneration,
        catalog_digest: CatalogDigest,
        usage: RetentionClosureUsage,
        digest: RetentionClosureDigest,
    ) -> Self {
        Self {
            profile,
            catalog_generation,
            catalog_digest,
            usage,
            digest,
        }
    }
}
