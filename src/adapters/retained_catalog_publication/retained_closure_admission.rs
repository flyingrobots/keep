//! This module owns proof that every current retained closure was admitted.

use super::PreservedRetentionManifest;
use crate::RetentionHead;

/// Proof that one call admitted current retention evidence for its candidate.
///
/// Only [`publish_retained_catalog_generation`](super::publish_retained_catalog_generation)
/// creates this value, after it has admitted the current retention head, the
/// manifest it names, and every root that manifest names, and has verified each
/// root's closure against the candidate catalog. It is neither `Clone` nor
/// constructible outside the gate, so holding a reference proves the gate ran
/// for the current call.
#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct RetainedClosureAdmission {
    head: Option<RetentionHead>,
    root_count: u32,
}

impl RetainedClosureAdmission {
    /// Returns the preserved manifest, absent when no retention head exists.
    #[must_use]
    pub const fn preserved_manifest(&self) -> Option<PreservedRetentionManifest> {
        match self.head {
            Some(head) => Some(PreservedRetentionManifest::new(
                head.generation(),
                head.manifest_digest(),
                self.root_count,
            )),
            None => None,
        }
    }

    pub(super) const fn unretained() -> Self {
        Self {
            head: None,
            root_count: 0,
        }
    }

    pub(super) const fn retained(head: RetentionHead, root_count: u32) -> Self {
        Self {
            head: Some(head),
            root_count,
        }
    }

    pub(super) const fn head(&self) -> Option<RetentionHead> {
        self.head
    }
}
