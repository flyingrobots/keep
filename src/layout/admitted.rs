//! Admitted semantic flat layout.

use super::{LayoutEntry, LayoutEntryLimit, LayoutValidationError, validation::validate_entries};
use crate::{BlobId, ChunkSpan, RegisteredStorageProfile};

/// A structurally valid flat layout under one registered storage profile.
///
/// Construction materializes one [`LayoutEntry`] per supplied span. It
/// performs no I/O and does not retain source bytes. Admission proves
/// structure and resource policy, not possession or content verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedLayout {
    target: BlobId,
    profile: RegisteredStorageProfile,
    entries: Box<[LayoutEntry]>,
}

impl AdmittedLayout {
    /// Admits exact detector spans for one target blob and registered profile.
    ///
    /// This consumes and materializes `spans`, with memory proportional to the
    /// admitted entry count and bounded by `entry_limit`.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutValidationError`] for resource-cap, allocation,
    /// cardinality, profile-bound, ordering, overflow, or aggregate failures.
    pub fn from_spans(
        target: BlobId,
        profile: RegisteredStorageProfile,
        spans: Vec<ChunkSpan>,
        entry_limit: LayoutEntryLimit,
    ) -> Result<Self, LayoutValidationError> {
        check_entry_limit(spans.len(), entry_limit)?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(spans.len())
            .map_err(|source| LayoutValidationError::Allocation { source })?;
        for span in spans {
            entries.push(LayoutEntry::from(span));
        }
        validate_entries(target, profile, &entries, entry_limit)?;
        Ok(Self::from_validated_entries(target, profile, entries))
    }

    pub(super) fn from_validated_entries(
        target: BlobId,
        profile: RegisteredStorageProfile,
        entries: Vec<LayoutEntry>,
    ) -> Self {
        Self {
            target,
            profile,
            entries: entries.into_boxed_slice(),
        }
    }

    /// Returns the exact target logical blob identity.
    #[must_use]
    pub const fn target(&self) -> BlobId {
        self.target
    }

    /// Returns the registered deterministic storage profile.
    #[must_use]
    pub const fn profile(&self) -> RegisteredStorageProfile {
        self.profile
    }

    /// Returns the canonical ordered semantic entries.
    #[must_use]
    pub const fn entries(&self) -> &[LayoutEntry] {
        &self.entries
    }
}

#[allow(
    clippy::redundant_pub_crate,
    reason = "the reference adapter enforces this domain admission law while streaming"
)]
pub(crate) fn check_entry_limit(
    observed: usize,
    limit: LayoutEntryLimit,
) -> Result<(), LayoutValidationError> {
    let maximum = usize::try_from(limit.get()).map_err(|_source| {
        LayoutValidationError::EntryLimitHostWidth {
            observed: limit.get(),
        }
    })?;
    if observed > maximum {
        return Err(LayoutValidationError::EntryLimitExceeded {
            maximum: limit.get(),
            observed,
        });
    }
    Ok(())
}
