//! This boundary module owns candidate-to-manifest successor binding.

use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, RetentionPublicationPreparationError,
    manifest_entry_update::{self, ManifestEntryUpdate},
};
use crate::{LivenessGeneration, RetentionManifest, RetentionManifestEntry, RootGeneration};

pub(super) fn build(
    candidate: &AdmittedRetentionRoot<'_>,
    current: Option<&AdmittedRetentionManifest<'_>>,
) -> Result<RetentionManifest, RetentionPublicationPreparationError> {
    let entry = candidate_entry(candidate);
    let Some(current) = current else {
        require_initial(candidate)?;
        let entries =
            manifest_entry_update::apply(&[], ManifestEntryUpdate::Insert { index: 0, entry })?;
        return RetentionManifest::new(LivenessGeneration::INITIAL, None, entries)
            .map_err(|source| RetentionPublicationPreparationError::Manifest { source });
    };
    let generation = current
        .manifest()
        .generation()
        .successor()
        .map_err(|source| RetentionPublicationPreparationError::LivenessGeneration { source })?;
    let namespace = entry.namespace();
    let entries = current.manifest().entries();
    let update = match entries.binary_search_by_key(&namespace, |item| item.namespace()) {
        Ok(index) => {
            let selected = entries.get(index).copied().ok_or(
                RetentionPublicationPreparationError::ManifestEntryIndex {
                    index,
                    entry_count: entries.len(),
                },
            )?;
            require_successor(selected, candidate)?;
            ManifestEntryUpdate::Replace { index, entry }
        }
        Err(index) => {
            require_initial(candidate)?;
            ManifestEntryUpdate::Insert { index, entry }
        }
    };
    let entries = manifest_entry_update::apply(entries, update)?;
    RetentionManifest::new(generation, Some(current.digest()), entries)
        .map_err(|source| RetentionPublicationPreparationError::Manifest { source })
}

pub(super) fn require_current_selection<'borrow, 'encoded>(
    candidate: &AdmittedRetentionRoot<'_>,
    current: Option<&'borrow AdmittedRetentionManifest<'encoded>>,
) -> Result<&'borrow AdmittedRetentionManifest<'encoded>, RetentionPublicationPreparationError> {
    let namespace = candidate.root().namespace().digest();
    let current = current
        .ok_or(RetentionPublicationPreparationError::CurrentManifestRequired { namespace })?;
    let entry = current
        .manifest()
        .entries()
        .binary_search_by_key(&namespace, |item| item.namespace())
        .ok()
        .and_then(|index| current.manifest().entries().get(index))
        .copied()
        .ok_or_else(
            || RetentionPublicationPreparationError::CurrentManifestEntryMissing {
                namespace,
                generation: candidate.root().generation(),
                digest: candidate.digest(),
            },
        )?;
    if entry.root_generation() == candidate.root().generation()
        && entry.root_digest() == candidate.digest()
    {
        Ok(current)
    } else {
        Err(current_mismatch(entry, candidate))
    }
}

fn require_initial(
    candidate: &AdmittedRetentionRoot<'_>,
) -> Result<(), RetentionPublicationPreparationError> {
    if candidate.root().generation() == RootGeneration::INITIAL
        && candidate.root().predecessor().is_none()
    {
        Ok(())
    } else {
        Err(
            RetentionPublicationPreparationError::UnexpectedNamespaceSuccessor {
                namespace: candidate.root().namespace().digest(),
                generation: candidate.root().generation(),
                predecessor: candidate.root().predecessor(),
            },
        )
    }
}

fn require_successor(
    current: RetentionManifestEntry,
    candidate: &AdmittedRetentionRoot<'_>,
) -> Result<(), RetentionPublicationPreparationError> {
    let expected_generation = current
        .root_generation()
        .successor()
        .map_err(|source| RetentionPublicationPreparationError::RootGeneration { source })?;
    if candidate.root().generation() == expected_generation
        && candidate.root().predecessor() == Some(current.root_digest())
    {
        Ok(())
    } else {
        Err(successor_mismatch(current, candidate))
    }
}

fn candidate_entry(candidate: &AdmittedRetentionRoot<'_>) -> RetentionManifestEntry {
    RetentionManifestEntry::new(
        candidate.root().namespace().digest(),
        candidate.root().generation(),
        candidate.digest(),
    )
}

const fn successor_mismatch(
    current: RetentionManifestEntry,
    candidate: &AdmittedRetentionRoot<'_>,
) -> RetentionPublicationPreparationError {
    RetentionPublicationPreparationError::ManifestSuccessorMismatch {
        namespace: current.namespace(),
        current_generation: current.root_generation(),
        current_digest: current.root_digest(),
        candidate_generation: candidate.root().generation(),
        candidate_predecessor: candidate.root().predecessor(),
    }
}

const fn current_mismatch(
    current: RetentionManifestEntry,
    candidate: &AdmittedRetentionRoot<'_>,
) -> RetentionPublicationPreparationError {
    RetentionPublicationPreparationError::CurrentManifestEntryMismatch {
        namespace: current.namespace(),
        current_generation: current.root_generation(),
        current_digest: current.root_digest(),
        candidate_generation: candidate.root().generation(),
        candidate_digest: candidate.digest(),
    }
}
