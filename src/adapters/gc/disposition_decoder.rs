//! This boundary module owns recovery-disposition receipt decoding order:
//! framing, checksum, then every registered enumeration and coordinate.

use super::RecoveryDispositionDecodeError as Error;
use super::{
    AdmittedRecoveryDispositionReceipt, ArtifactContentDigest, ArtifactIdentityDigest,
    DecisionEvidenceDigest, GcRetentionState, ObservedHeadChecksum, ReaderLockIdentity,
    RecoveryArtifactKind, RecoveryClassification, RecoveryDispositionArtifact,
    RecoveryDispositionCoordinates, RecoveryDispositionDecision, RecoveryDispositionField,
    RecoveryDispositionReceipt, disposition_format as format,
};
use crate::{CatalogDigest, CatalogGeneration, LivenessGeneration, RetentionManifestDigest};

pub(super) fn decode(encoded: &[u8]) -> Result<AdmittedRecoveryDispositionReceipt<'_>, Error> {
    require_length(encoded)?;
    validate_fixed_fields(encoded)?;
    verify_checksum(encoded)?;
    let kind = registered(
        encoded,
        24,
        RecoveryDispositionField::ArtifactKind,
        RecoveryArtifactKind::from_code,
    )?;
    let decision = registered(
        encoded,
        26,
        RecoveryDispositionField::Decision,
        RecoveryDispositionDecision::from_code,
    )?;
    let classification = registered(
        encoded,
        28,
        RecoveryDispositionField::Classification,
        RecoveryClassification::from_code,
    )?;
    let artifact = RecoveryDispositionArtifact {
        kind,
        classification,
        length: read_u64(encoded, 32)?,
        identity_digest: ArtifactIdentityDigest::new(read_array(encoded, 40)?),
        content_digest: ArtifactContentDigest::new(read_array(encoded, 72)?),
    };
    let coordinates = RecoveryDispositionCoordinates {
        publication_generation: catalog_generation(encoded, 104)?,
        publication_checksum: ObservedHeadChecksum::new(read_array(encoded, 112)?),
        catalog_generation: catalog_generation(encoded, 144)?,
        catalog_digest: CatalogDigest::from_validated(read_array(encoded, 152)?),
        retention: retention_state(encoded)?,
        reader_lock: ReaderLockIdentity::new(
            read_u64(encoded, 224)?,
            read_u64(encoded, 232)?,
            read_u64(encoded, 240)?,
        ),
    };
    let evidence_digest = DecisionEvidenceDigest::new(read_array(encoded, 248)?);
    let receipt = RecoveryDispositionReceipt::new(artifact, decision, coordinates, evidence_digest);
    Ok(AdmittedRecoveryDispositionReceipt::admitted(
        encoded, &receipt,
    ))
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
    if read_u16(encoded, format::HEADER_RESERVED_OFFSET)? != 0 {
        return Err(Error::NonZeroReserved);
    }
    let trailer: [u8; format::TRAILER_RESERVED_LENGTH] =
        read_array(encoded, format::TRAILER_RESERVED_OFFSET)?;
    if trailer != [0_u8; format::TRAILER_RESERVED_LENGTH] {
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

fn registered<T>(
    encoded: &[u8],
    offset: usize,
    field: RecoveryDispositionField,
    admit: fn(u16) -> Option<T>,
) -> Result<T, Error> {
    let observed = read_u16(encoded, offset)?;
    admit(observed).ok_or(Error::UnregisteredCode { field, observed })
}

/// Liveness generation zero names the canonical empty retention state and
/// must carry its digest; any positive generation names a published head.
fn retention_state(encoded: &[u8]) -> Result<GcRetentionState, Error> {
    let generation = read_u64(encoded, 184)?;
    let digest: [u8; 32] = read_array(encoded, 192)?;
    match LivenessGeneration::new(generation) {
        Ok(generation) => Ok(GcRetentionState::Published {
            generation,
            manifest_digest: RetentionManifestDigest::from_hash(digest),
        }),
        Err(_zero) if digest == format::empty_retention_digest() => Ok(GcRetentionState::Empty),
        Err(_zero) => Err(Error::EmptyRetentionDigestMismatch { observed: digest }),
    }
}

fn catalog_generation(encoded: &[u8], offset: usize) -> Result<CatalogGeneration, Error> {
    CatalogGeneration::new(read_u64(encoded, offset)?)
        .map_err(|_source| Error::ZeroGeneration { offset })
}

const fn require_length(encoded: &[u8]) -> Result<(), Error> {
    if encoded.len() == format::ENCODED_LENGTH {
        Ok(())
    } else {
        Err(wrong_length(encoded))
    }
}

const fn wrong_length(encoded: &[u8]) -> Error {
    Error::WrongLength {
        expected: format::ENCODED_LENGTH,
        observed: encoded.len(),
    }
}

fn read_u16(encoded: &[u8], offset: usize) -> Result<u16, Error> {
    read_array(encoded, offset).map(u16::from_be_bytes)
}

fn read_u32(encoded: &[u8], offset: usize) -> Result<u32, Error> {
    read_array(encoded, offset).map(u32::from_be_bytes)
}

fn read_u64(encoded: &[u8], offset: usize) -> Result<u64, Error> {
    read_array(encoded, offset).map(u64::from_be_bytes)
}

fn read_array<const WIDTH: usize>(encoded: &[u8], offset: usize) -> Result<[u8; WIDTH], Error> {
    let Some(end) = offset.checked_add(WIDTH) else {
        return Err(wrong_length(encoded));
    };
    let bytes = encoded
        .get(offset..end)
        .ok_or_else(|| wrong_length(encoded))?;
    <[u8; WIDTH]>::try_from(bytes).map_err(|_| wrong_length(encoded))
}
