//! This module owns the evidence one disposition binds: the disposed
//! artifact, its pool identity, its receipt, and the residue reads.

use std::io::{self, Read};

use cap_std::fs::Dir;

use super::filesystem_retention_current::read_exact_optional;
use super::filesystem_retention_disposition::{
    FilesystemRetentionDispositionError, PoolEntry, invalid_data_from, observe,
};
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_recovery_observation::RetentionRecoveryObservation;
use super::filesystem_retention_stage::invalid_data;
use super::{AdmittedRetentionManifest, AdmittedRetentionRoot, RecoveryDispositionTarget};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record as exact_record;
use crate::adapters::{
    ArtifactIdentityDigest, CanonicalRecoveryDispositionReceipt, DecisionEvidenceDigest,
    RecoveryArtifactKind, RecoveryClassification, RecoveryDispositionArtifact,
    RecoveryDispositionCoordinates, RecoveryDispositionDecision, RecoveryDispositionReceipt,
};

const RECEIPT_LENGTH: usize = 320;

/// A directory entry name with its complete bytes.
type NamedEntry = (String, Box<[u8]>);

/// The retained stage's exact bytes and the pool entry recovery linked them to.
pub(super) fn disposed_artifact(
    observation: &RetentionRecoveryObservation,
    target: RecoveryDispositionTarget,
) -> io::Result<(Box<[u8]>, PoolEntry)> {
    match target {
        RecoveryDispositionTarget::Root => {
            let bytes = observation
                .root()
                .ok_or_else(|| {
                    invalid_data(super::FilesystemRetentionStageRefusal::DispositionRootStageAbsent)
                })?
                .bytes
                .clone();
            let root = AdmittedRetentionRoot::decode(&bytes).map_err(invalid_data_from)?;
            let pool = PoolEntry::Root {
                namespace: pool_name::namespace(root.root().namespace().digest()),
                name: pool_name::root(root.root().generation(), root.digest()),
            };
            Ok((bytes, pool))
        }
        RecoveryDispositionTarget::Manifest => {
            let bytes = observation
                .manifest()
                .ok_or_else(|| {
                    invalid_data(
                        super::FilesystemRetentionStageRefusal::DispositionManifestStageAbsent,
                    )
                })?
                .bytes
                .clone();
            let manifest = AdmittedRetentionManifest::decode(&bytes).map_err(invalid_data_from)?;
            let pool = PoolEntry::Manifest {
                name: pool_name::manifest(manifest.manifest().generation(), manifest.digest()),
            };
            Ok((bytes, pool))
        }
    }
}

/// The artifact's pool-name digest: the identity the receipt is filed under.
pub(super) fn pool_identity(
    pool: &PoolEntry,
    artifact: &[u8],
) -> Result<(RecoveryArtifactKind, ArtifactIdentityDigest), FilesystemRetentionDispositionError> {
    match pool {
        PoolEntry::Root { .. } => {
            let root = AdmittedRetentionRoot::decode(artifact)
                .map_err(|source| observe(invalid_data_from(source)))?;
            Ok((
                RecoveryArtifactKind::RetentionRoot,
                ArtifactIdentityDigest::new(*root.digest().as_bytes()),
            ))
        }
        PoolEntry::Manifest { .. } => {
            let manifest = AdmittedRetentionManifest::decode(artifact)
                .map_err(|source| observe(invalid_data_from(source)))?;
            Ok((
                RecoveryArtifactKind::RetentionManifest,
                ArtifactIdentityDigest::new(*manifest.digest().as_bytes()),
            ))
        }
    }
}

pub(super) fn receipt_for(
    artifact: &[u8],
    (kind, identity): (RecoveryArtifactKind, ArtifactIdentityDigest),
    decision: RecoveryDispositionDecision,
    coordinates: RecoveryDispositionCoordinates,
) -> io::Result<CanonicalRecoveryDispositionReceipt> {
    let disposed = RecoveryDispositionArtifact {
        kind,
        classification: RecoveryClassification::CompleteOrphan,
        length: u64::try_from(artifact.len()).map_err(invalid_data_from)?,
        identity_digest: identity,
        content_digest: CanonicalRecoveryDispositionReceipt::artifact_content_digest(artifact),
    };
    let evidence = DecisionEvidenceDigest::new(trailing_checksum(artifact)?);
    Ok(CanonicalRecoveryDispositionReceipt::from_receipt(
        &RecoveryDispositionReceipt::new(disposed, decision, coordinates, evidence),
    ))
}

/// The artifact record's trailing checksum: the evidence the decision was
/// made over.
pub(super) fn trailing_checksum(bytes: &[u8]) -> io::Result<[u8; 32]> {
    let start = bytes.len().checked_sub(32).ok_or_else(|| {
        invalid_data(super::FilesystemRetentionStageRefusal::ArtifactChecksumLength)
    })?;
    bytes
        .get(start..)
        .and_then(|slice| slice.try_into().ok())
        .ok_or_else(|| invalid_data(super::FilesystemRetentionStageRefusal::ArtifactChecksumSlot))
}

/// The first regular entry of `directory` whose name ends with `suffix`,
/// with its complete bytes.
pub(super) fn entry_with_suffix(directory: &Dir, suffix: &str) -> io::Result<Option<NamedEntry>> {
    for entry in directory.entries()? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        if !name.ends_with(suffix) {
            continue;
        }
        let metadata = directory.symlink_metadata(&name)?;
        if !metadata.is_file() {
            return Err(invalid_data(
                super::FilesystemRetentionStageRefusal::PoolEntryKind,
            ));
        }
        let length = usize::try_from(metadata.len()).map_err(invalid_data_from)?;
        let bytes = read_exact_optional(directory, &name, length)?.ok_or_else(|| {
            invalid_data(super::FilesystemRetentionStageRefusal::PoolEntryVanished)
        })?;
        return Ok(Some((name, bytes)));
    }
    Ok(None)
}

/// Reads `recovery/disposition.next` up to one byte past the receipt length.
pub(super) fn read_bounded(recovery: &Dir, name: &str) -> io::Result<Option<Vec<u8>>> {
    let mut file = match exact_record::open_read(recovery, name) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(source),
    };
    if !file.metadata()?.is_file() {
        return Err(invalid_data(
            super::FilesystemRetentionStageRefusal::DispositionStageKind,
        ));
    }
    let mut bytes = Vec::new();
    let limit = u64::try_from(RECEIPT_LENGTH)
        .map_err(invalid_data_from)?
        .saturating_add(1);
    file.by_ref().take(limit).read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}

pub(super) fn discard_stage(recovery: &Dir) -> io::Result<()> {
    recovery.remove_file(pool_name::DISPOSITION_STAGE)?;
    exact_record::require_absent(recovery, pool_name::DISPOSITION_STAGE)
        .map_err(exact_record::ExactRecordError::into_io)?;
    synchronize_directory(recovery)
}
