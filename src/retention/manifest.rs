//! This module owns one canonical semantic global retention manifest.

use super::{
    LivenessGeneration, RetentionManifestDigest, RetentionManifestEntry, RetentionManifestError,
};

/// Complete namespace-to-root view at one global liveness generation.
///
/// Entries are stored in strict namespace-digest order. Construction sorts
/// caller input, refuses duplicate namespaces, and allocates no additional
/// buffer beyond the supplied `Vec`.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionManifest {
    generation: LivenessGeneration,
    predecessor: Option<RetentionManifestDigest>,
    entries: Vec<RetentionManifestEntry>,
    entry_count: u32,
}

impl RetentionManifest {
    /// Maximum admitted namespace entries.
    pub const MAXIMUM_ENTRY_COUNT: u32 = 4_096;

    /// Admits one complete semantic manifest.
    ///
    /// # Errors
    ///
    /// Returns a typed generation-history, entry-count, or duplicate-namespace
    /// failure before the value is admitted.
    pub fn new(
        generation: LivenessGeneration,
        predecessor: Option<RetentionManifestDigest>,
        mut entries: Vec<RetentionManifestEntry>,
    ) -> Result<Self, RetentionManifestError> {
        admit_predecessor(generation, predecessor)?;
        let observed = entries.len();
        let entry_count =
            u32::try_from(observed).map_err(|_| RetentionManifestError::EntryCountExceeded {
                maximum: Self::MAXIMUM_ENTRY_COUNT,
                observed,
            })?;
        if entry_count > Self::MAXIMUM_ENTRY_COUNT {
            return Err(RetentionManifestError::EntryCountExceeded {
                maximum: Self::MAXIMUM_ENTRY_COUNT,
                observed,
            });
        }
        entries.sort_unstable_by_key(|entry| entry.namespace());
        refuse_duplicate(&entries)?;
        Ok(Self {
            generation,
            predecessor,
            entries,
            entry_count,
        })
    }

    /// Returns the exact global liveness generation.
    pub const fn generation(&self) -> LivenessGeneration {
        self.generation
    }

    /// Returns the preceding manifest digest, if this is a successor.
    pub const fn predecessor(&self) -> Option<RetentionManifestDigest> {
        self.predecessor
    }

    /// Returns entries in strict namespace-digest order.
    pub fn entries(&self) -> &[RetentionManifestEntry] {
        &self.entries
    }

    /// Returns the bounded entry count.
    pub const fn entry_count(&self) -> u32 {
        self.entry_count
    }
}

fn admit_predecessor(
    generation: LivenessGeneration,
    predecessor: Option<RetentionManifestDigest>,
) -> Result<(), RetentionManifestError> {
    if generation.get() == 1 {
        return predecessor.map_or(Ok(()), |observed| {
            Err(RetentionManifestError::InitialGenerationHasPredecessor { observed })
        });
    }
    if predecessor.is_some() {
        Ok(())
    } else {
        Err(RetentionManifestError::MissingPredecessor { generation })
    }
}

fn refuse_duplicate(entries: &[RetentionManifestEntry]) -> Result<(), RetentionManifestError> {
    for pair in entries.windows(2) {
        let [previous, observed] = pair else {
            continue;
        };
        if previous.namespace() == observed.namespace() {
            return Err(RetentionManifestError::DuplicateNamespace {
                namespace: previous.namespace(),
            });
        }
    }
    Ok(())
}
