//! This boundary module owns bounded retention manifest entry updates.

use super::RetentionPublicationPreparationError;
use crate::{RetentionManifest, RetentionManifestEntry, RetentionManifestError};

#[derive(Clone, Copy)]
pub(super) enum ManifestEntryUpdate {
    Insert {
        index: usize,
        entry: RetentionManifestEntry,
    },
    Replace {
        index: usize,
        entry: RetentionManifestEntry,
    },
}

pub(super) fn apply(
    current: &[RetentionManifestEntry],
    update: ManifestEntryUpdate,
) -> Result<Vec<RetentionManifestEntry>, RetentionPublicationPreparationError> {
    let inserts = usize::from(matches!(update, ManifestEntryUpdate::Insert { .. }));
    let observed = current.len().checked_add(inserts).ok_or(
        RetentionPublicationPreparationError::Manifest {
            source: RetentionManifestError::EntryCountExceeded {
                maximum: RetentionManifest::MAXIMUM_ENTRY_COUNT,
                observed: usize::MAX,
            },
        },
    )?;
    require_admitted_count(observed)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(observed)
        .map_err(|source| RetentionPublicationPreparationError::EntryAllocation { source })?;
    let (index, entry, skip) = match update {
        ManifestEntryUpdate::Insert { index, entry } => (index, entry, 0),
        ManifestEntryUpdate::Replace { index, entry } => (index, entry, 1),
    };
    let (before, remaining) = split(current, index)?;
    let after =
        remaining
            .get(skip..)
            .ok_or(RetentionPublicationPreparationError::ManifestEntryIndex {
                index,
                entry_count: current.len(),
            })?;
    entries.extend_from_slice(before);
    entries.push(entry);
    entries.extend_from_slice(after);
    Ok(entries)
}

fn require_admitted_count(observed: usize) -> Result<(), RetentionPublicationPreparationError> {
    let admitted = u32::try_from(observed).map_err(|_| entry_count_error(observed))?;
    if admitted > RetentionManifest::MAXIMUM_ENTRY_COUNT {
        Err(entry_count_error(observed))
    } else {
        Ok(())
    }
}

fn split(
    current: &[RetentionManifestEntry],
    index: usize,
) -> Result<
    (&[RetentionManifestEntry], &[RetentionManifestEntry]),
    RetentionPublicationPreparationError,
> {
    current.split_at_checked(index).ok_or(
        RetentionPublicationPreparationError::ManifestEntryIndex {
            index,
            entry_count: current.len(),
        },
    )
}

const fn entry_count_error(observed: usize) -> RetentionPublicationPreparationError {
    RetentionPublicationPreparationError::Manifest {
        source: RetentionManifestError::EntryCountExceeded {
            maximum: RetentionManifest::MAXIMUM_ENTRY_COUNT,
            observed,
        },
    }
}
