//! This module owns admission of current retention evidence for one candidate.
//!
//! Storage supplies exact bytes; this module decodes and cross-checks them so
//! evidence admission never depends on the storage being checked. Closure
//! work reuses the storage-independent [`verify_retention_closure`].

use super::retained_evidence_source::RetainedEvidenceSource;
use super::{
    RetainedCatalogPublicationError as Error, RetainedClosureAdmission,
    RetentionEvidencePhase as Phase,
};
use crate::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CatalogSnapshot, ChecksummedRetentionHead,
    RetentionHead, RetentionManifestEntry, verify_retention_closure,
};

/// Admits current retention evidence and verifies every closure against `candidate`.
///
/// Roots are read, admitted, and verified one at a time in manifest order, so
/// at most one root's bytes are held. The first refusal is returned.
pub(super) fn admit(
    storage: &mut impl RetainedEvidenceSource,
    candidate: &CatalogSnapshot<'_, '_, '_>,
) -> Result<RetainedClosureAdmission, Error> {
    let Some(head) = read_head(storage, Phase::AdmitHead)? else {
        return Ok(RetainedClosureAdmission::unretained());
    };
    let encoded =
        storage
            .read_retention_manifest(&head)
            .map_err(|source| Error::RetentionStorage {
                phase: Phase::ReadManifest,
                source,
            })?;
    let manifest = admit_manifest(&head, &encoded)?;
    for entry in manifest.manifest().entries().iter().copied() {
        verify_root(storage, entry, candidate)?;
    }
    Ok(RetainedClosureAdmission::retained(
        head,
        manifest.manifest().entry_count(),
    ))
}

/// Requires the current retention head to equal the admitted head.
///
/// Head decoding is canonical, so semantic equality is byte equality. Manifest
/// and root pool entries are immutable and named by digest, so an unchanged
/// head selects exactly the admitted manifest and roots.
pub(super) fn revalidate(
    storage: &mut impl RetainedEvidenceSource,
    admission: &RetainedClosureAdmission,
) -> Result<(), Error> {
    let expected = admission.head();
    let observed = read_head(storage, Phase::RevalidateHead)?;
    if observed == expected {
        Ok(())
    } else {
        Err(Error::RetentionChanged {
            expected,
            observed: Box::new(observed),
        })
    }
}

fn read_head(
    storage: &mut impl RetainedEvidenceSource,
    phase: Phase,
) -> Result<Option<RetentionHead>, Error> {
    let Some(encoded) = storage
        .read_retention_head()
        .map_err(|source| Error::RetentionStorage { phase, source })?
    else {
        return Ok(None);
    };
    let checksummed = ChecksummedRetentionHead::decode(&encoded)
        .map_err(|source| Error::HeadDecode { phase, source })?;
    Ok(Some(*checksummed.head()))
}

fn admit_manifest<'encoded>(
    head: &RetentionHead,
    encoded: &'encoded [u8],
) -> Result<AdmittedRetentionManifest<'encoded>, Error> {
    let expected = head.manifest_length();
    if u64::try_from(encoded.len()).ok() != Some(expected.get()) {
        return Err(Error::ManifestLength {
            expected,
            observed: encoded.len(),
        });
    }
    let admitted =
        AdmittedRetentionManifest::decode(encoded).map_err(|source| Error::ManifestDecode {
            source: Box::new(source),
        })?;
    require_manifest_binding(head, &admitted)?;
    Ok(admitted)
}

fn require_manifest_binding(
    head: &RetentionHead,
    admitted: &AdmittedRetentionManifest<'_>,
) -> Result<(), Error> {
    let manifest = admitted.manifest();
    if admitted.digest() != head.manifest_digest() {
        return Err(Error::ManifestDigest {
            expected: head.manifest_digest(),
            observed: admitted.digest(),
        });
    }
    if manifest.generation() != head.generation() {
        return Err(Error::ManifestGeneration {
            expected: head.generation(),
            observed: manifest.generation(),
        });
    }
    if manifest.predecessor() != head.predecessor() {
        return Err(Error::ManifestPredecessor {
            expected: head.predecessor(),
            observed: manifest.predecessor(),
        });
    }
    Ok(())
}

fn verify_root(
    storage: &mut impl RetainedEvidenceSource,
    entry: RetentionManifestEntry,
    candidate: &CatalogSnapshot<'_, '_, '_>,
) -> Result<(), Error> {
    let encoded = storage
        .read_retained_root(entry)
        .map_err(|source| Error::RootStorage { entry, source })?;
    let admitted = AdmittedRetentionRoot::decode(&encoded).map_err(|source| Error::RootDecode {
        entry,
        source: Box::new(source),
    })?;
    let root = admitted.root();
    let observed = RetentionManifestEntry::new(
        root.namespace().digest(),
        root.generation(),
        admitted.digest(),
    );
    if observed != entry {
        return Err(Error::RootBinding {
            expected: entry,
            observed: Box::new(observed),
        });
    }
    let _verified = verify_retention_closure(root, candidate).map_err(|source| Error::Closure {
        entry,
        source: Box::new(source),
    })?;
    Ok(())
}
