//! This boundary module owns GC retirement receipt decoding order.

use super::GcRetirementReceiptDecodeError as Error;
use super::receipt::GcRetirementReceiptFields;
use super::receipt_bytes::{
    read_array, read_u16, read_u32, read_u64, require_length, wrong_length,
};
use super::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, GcCandidateSetDigest,
    GcRetirementIntentDigest, GcRetirementReceipt, PoolStateDigest, ReaderLockCoordinate,
    ReaderLockDevice, ReaderLockFile, ReaderLockIdentity, ReaderLockMount,
    receipt_format as format,
};
use crate::{
    CatalogDigest, CatalogGeneration, GcGeneration, LivenessGeneration, RetentionManifestDigest,
};

/// Decodes one receipt's framing, checksum, and every field without an
/// intent to bind it to: the prior retirement's receipt a new execution
/// succeeds, or the completion a fresh restart reports.
pub(super) fn decode_unbound(encoded: &[u8]) -> Result<GcRetirementReceipt, Error> {
    require_length(encoded)?;
    validate_fixed_fields(encoded)?;
    verify_checksum(encoded)?;
    let generation = GcGeneration::new(read_u64(encoded, 24)?)
        .map_err(|_source| Error::ZeroGeneration { offset: 24 })?;
    let liveness_generation = LivenessGeneration::new(read_u64(encoded, 128)?)
        .map_err(|_source| Error::ZeroGeneration { offset: 128 })?;
    let catalog_generation = CatalogGeneration::new(read_u64(encoded, 168)?)
        .map_err(|_source| Error::ZeroGeneration { offset: 168 })?;
    Ok(GcRetirementReceipt::from_fields(
        GcRetirementReceiptFields {
            generation,
            intent_digest: GcRetirementIntentDigest::from_hash(read_array(encoded, 32)?),
            retired_candidate_set_digest: GcCandidateSetDigest::from_verified(read_array(
                encoded, 64,
            )?),
            pool_state_digest: PoolStateDigest::new(read_array(encoded, 96)?),
            liveness_generation,
            manifest_digest: RetentionManifestDigest::from_hash(read_array(encoded, 136)?),
            catalog_generation,
            catalog_digest: CatalogDigest::from_validated(read_array(encoded, 176)?),
            reader_lock: ReaderLockIdentity::new(
                ReaderLockDevice::new(read_u64(encoded, 208)?),
                ReaderLockMount::new(read_u64(encoded, 216)?),
                ReaderLockFile::new(read_u64(encoded, 224)?),
            ),
            synchronization_count: read_u64(encoded, 232)?,
        },
    ))
}

pub(super) fn decode<'encoded>(
    encoded: &'encoded [u8],
    intent: &AdmittedGcRetirementIntent<'_>,
) -> Result<AdmittedGcRetirementReceipt<'encoded>, Error> {
    require_length(encoded)?;
    validate_fixed_fields(encoded)?;
    verify_checksum(encoded)?;
    let coordinates = intent.intent().coordinates();
    require_u64(
        encoded,
        24,
        coordinates.generation.get(),
        |expected, observed| Error::GenerationMismatch { expected, observed },
    )?;
    require_digest(
        encoded,
        32,
        intent.digest().as_bytes(),
        |expected, observed| Error::IntentDigestMismatch { expected, observed },
    )?;
    require_digest(
        encoded,
        64,
        intent.candidate_set_digest().as_bytes(),
        |expected, observed| Error::RetiredSetDigestMismatch { expected, observed },
    )?;
    let pool_state_digest = PoolStateDigest::new(read_array(encoded, 96)?);
    require_u64(
        encoded,
        128,
        coordinates.liveness_generation.get(),
        |expected, observed| Error::LivenessGenerationMismatch { expected, observed },
    )?;
    require_digest(
        encoded,
        136,
        coordinates.manifest_digest.as_bytes(),
        |expected, observed| Error::ManifestDigestMismatch { expected, observed },
    )?;
    require_u64(
        encoded,
        168,
        coordinates.catalog_generation.get(),
        |expected, observed| Error::CatalogGenerationMismatch { expected, observed },
    )?;
    require_digest(
        encoded,
        176,
        coordinates.catalog_digest.as_bytes(),
        |expected, observed| Error::CatalogDigestMismatch { expected, observed },
    )?;
    require_reader_lock(encoded, intent)?;
    require_u64(
        encoded,
        232,
        u64::from(intent.intent().candidate_count()),
        |expected, observed| Error::SynchronizationCountMismatch { expected, observed },
    )?;
    let receipt = GcRetirementReceipt::for_intent(
        intent.digest(),
        intent.candidate_set_digest(),
        intent.intent(),
        pool_state_digest,
    );
    Ok(AdmittedGcRetirementReceipt::admitted(encoded, receipt))
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), Error> {
    let magic = read_array(encoded, 0)?;
    if magic != format::MAGIC {
        return Err(Error::InvalidMagic { observed: magic });
    }
    let version = read_u16(encoded, 16)?;
    if version != format::VERSION {
        return Err(Error::UnsupportedVersion {
            expected: format::VERSION,
            observed: version,
        });
    }
    let record_length = read_u16(encoded, 18)?;
    if record_length != format::RECORD_LENGTH {
        return Err(Error::InvalidRecordLength {
            expected: format::RECORD_LENGTH,
            observed: record_length,
        });
    }
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(Error::UnsupportedFlags { observed: flags });
    }
    let reserved: [u8; format::RESERVED_LENGTH] = read_array(encoded, format::RESERVED_OFFSET)?;
    if reserved != [0_u8; format::RESERVED_LENGTH] {
        return Err(Error::NonZeroReserved);
    }
    Ok(())
}

fn verify_checksum(encoded: &[u8]) -> Result<(), Error> {
    let preimage = encoded
        .get(..format::CHECKSUM_OFFSET)
        .ok_or_else(|| wrong_length(encoded))?;
    let observed = read_array(encoded, format::CHECKSUM_OFFSET)?;
    let expected = format::checksum(preimage);
    if observed == expected {
        Ok(())
    } else {
        Err(Error::ChecksumMismatch { expected, observed })
    }
}

fn require_reader_lock(
    encoded: &[u8],
    intent: &AdmittedGcRetirementIntent<'_>,
) -> Result<(), Error> {
    let bound = intent.intent().coordinates().reader_lock;
    for (coordinate, offset, expected) in [
        (ReaderLockCoordinate::Device, 208, bound.device().get()),
        (ReaderLockCoordinate::Mount, 216, bound.mount().get()),
        (ReaderLockCoordinate::File, 224, bound.file().get()),
    ] {
        let observed = read_u64(encoded, offset)?;
        if observed != expected {
            return Err(Error::ReaderLockMismatch {
                coordinate,
                expected,
                observed,
            });
        }
    }
    Ok(())
}

fn require_u64<F>(encoded: &[u8], offset: usize, expected: u64, error: F) -> Result<(), Error>
where
    F: FnOnce(u64, u64) -> Error,
{
    let observed = read_u64(encoded, offset)?;
    if observed == expected {
        Ok(())
    } else {
        Err(error(expected, observed))
    }
}

fn require_digest<F>(
    encoded: &[u8],
    offset: usize,
    expected: &[u8; 32],
    error: F,
) -> Result<(), Error>
where
    F: FnOnce([u8; 32], [u8; 32]) -> Error,
{
    let observed: [u8; 32] = read_array(encoded, offset)?;
    if observed == *expected {
        Ok(())
    } else {
        Err(error(*expected, observed))
    }
}
