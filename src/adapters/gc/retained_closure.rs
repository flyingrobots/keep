//! This boundary module owns one retained root's physical closure.

use std::collections::BTreeSet;

use crate::adapters::SegmentDigest;
use crate::{
    RetentionClosureDigest, RetentionNamespaceDigest, RetentionRootDigest, RootGeneration,
};

/// The segments one retained root's verified closure reaches.
///
/// The closure digest is the verifier's own evidence; the segment set is the
/// physical projection of every record that closure resolved. A segment in
/// this set is live while this root is retained.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcRetainedClosure {
    namespace: RetentionNamespaceDigest,
    generation: RootGeneration,
    root_digest: RetentionRootDigest,
    closure_digest: RetentionClosureDigest,
    segments: BTreeSet<SegmentDigest>,
}

impl GcRetainedClosure {
    /// Binds one retained root to the segments its closure reaches.
    pub const fn new(
        namespace: RetentionNamespaceDigest,
        generation: RootGeneration,
        root_digest: RetentionRootDigest,
        closure_digest: RetentionClosureDigest,
        segments: BTreeSet<SegmentDigest>,
    ) -> Self {
        Self {
            namespace,
            generation,
            root_digest,
            closure_digest,
            segments,
        }
    }

    /// Returns the namespace the root retains.
    pub const fn namespace(&self) -> RetentionNamespaceDigest {
        self.namespace
    }

    /// Returns the retained root generation.
    pub const fn generation(&self) -> RootGeneration {
        self.generation
    }

    /// Returns the exact retained root digest.
    pub const fn root_digest(&self) -> RetentionRootDigest {
        self.root_digest
    }

    /// Returns the verifier's closure digest.
    pub const fn closure_digest(&self) -> RetentionClosureDigest {
        self.closure_digest
    }

    /// Returns every segment the closure reaches, in canonical order.
    #[must_use]
    pub const fn segments(&self) -> &BTreeSet<SegmentDigest> {
        &self.segments
    }
}
