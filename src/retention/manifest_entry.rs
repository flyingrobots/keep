//! This module owns one semantic retention manifest entry.

use super::{RetentionNamespaceDigest, RetentionRootDigest, RootGeneration};

/// Exact current root coordinate for one retention namespace.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetentionManifestEntry {
    namespace: RetentionNamespaceDigest,
    root_generation: RootGeneration,
    root_digest: RetentionRootDigest,
}

impl RetentionManifestEntry {
    /// Combines already-validated namespace and root coordinates.
    pub const fn new(
        namespace: RetentionNamespaceDigest,
        root_generation: RootGeneration,
        root_digest: RetentionRootDigest,
    ) -> Self {
        Self {
            namespace,
            root_generation,
            root_digest,
        }
    }

    /// Returns the namespace digest selected by this entry.
    pub const fn namespace(self) -> RetentionNamespaceDigest {
        self.namespace
    }

    /// Returns the exact current namespace root generation.
    pub const fn root_generation(self) -> RootGeneration {
        self.root_generation
    }

    /// Returns the exact current namespace root digest.
    pub const fn root_digest(self) -> RetentionRootDigest {
        self.root_digest
    }
}
