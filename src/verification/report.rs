//! This module owns read-only reports constructed after evidence admission.

use super::{VerificationDepth, VerificationSubject};
use crate::{CatalogDigest, CatalogGeneration};

/// Evidence established for one subject, with no public construction or upgrade.
///
/// Depth is a subject-specific claim, not authority to infer a stronger claim.
/// Copying this value preserves exactly the same evidence.
///
/// ```compile_fail
/// use keep::{VerificationDepth, VerifiedSubject};
/// fn deepen(mut subject: VerifiedSubject) {
///     subject.depth = VerificationDepth::CompleteBlobIdentity;
/// }
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerifiedSubject {
    subject: VerificationSubject,
    depth: VerificationDepth,
}

impl VerifiedSubject {
    /// Returns the exact subject whose evidence was admitted.
    pub const fn subject(&self) -> VerificationSubject {
        self.subject
    }

    /// Returns the depth established for this subject.
    pub const fn depth(&self) -> VerificationDepth {
        self.depth
    }
}

/// An immutable in-memory report from a successful verification operation.
///
/// The current operations each report one subject without allocating. A
/// report is not a durable receipt, reader fence, retention authority, or
/// assertion that an on-disk artifact still exists after the operation.
/// It contains no plaintext, keys, or filesystem paths.
///
/// Adapters may check more evidence during admission than the requested
/// depth; those costs are documented by each operation. Only the stated
/// per-subject evidence is certified by the report.
///
/// ```
/// use keep::{VerificationDepth, VerificationReport};
/// fn claimed_depths(report: &VerificationReport) -> Vec<VerificationDepth> {
///     report.subjects().iter().map(|subject| subject.depth()).collect()
/// }
/// ```
///
/// Callers cannot manufacture an upgraded report from subject coordinates.
///
/// ```compile_fail
/// use keep::{VerificationDepth, VerificationReport, VerificationSubject};
/// fn manufacture(subject: VerificationSubject) -> VerificationReport {
///     VerificationReport::established(subject, VerificationDepth::CompleteBlobIdentity)
/// }
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerificationReport {
    requested: VerificationDepth,
    catalog: Option<(CatalogGeneration, CatalogDigest)>,
    retention: Option<crate::RetentionHead>,
    subject: VerifiedSubject,
}

impl VerificationReport {
    /// Returns the original requested policy, without implicit escalation.
    pub const fn requested(&self) -> VerificationDepth {
        self.requested
    }

    /// Returns the evidence established for each subject in this report.
    pub const fn subjects(&self) -> &[VerifiedSubject] {
        std::slice::from_ref(&self.subject)
    }

    /// Returns the exact immutable catalog used by an operation, when applicable.
    ///
    /// This coordinate is evidence provenance, not a live fence or retention grant.
    pub const fn catalog(&self) -> Option<(CatalogGeneration, CatalogDigest)> {
        self.catalog
    }

    /// Returns the exact publication-selected retention head, when applicable.
    ///
    /// Root verification over supplied bytes alone has no publication coordinate.
    pub const fn retention_head(&self) -> Option<crate::RetentionHead> {
        self.retention
    }

    pub(crate) const fn in_retention(mut self, head: Option<crate::RetentionHead>) -> Self {
        self.retention = head;
        self
    }

    pub(crate) const fn in_catalog(
        mut self,
        generation: CatalogGeneration,
        digest: CatalogDigest,
    ) -> Self {
        self.catalog = Some((generation, digest));
        self
    }

    pub(crate) const fn established(
        subject: VerificationSubject,
        depth: VerificationDepth,
    ) -> Self {
        Self {
            requested: depth,
            catalog: None,
            retention: None,
            subject: VerifiedSubject { subject, depth },
        }
    }
}
