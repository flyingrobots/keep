//! This boundary module owns GC retirement intent header framing admission.

use super::GcRetirementIntentDecodeError as Error;
use super::intent_field_decoder::{
    read_array, read_u32, read_u64, require_exact, require_minimum, require_u16, require_zero,
};
use super::intent_format as format;

pub(super) struct DecodedIntentHeader {
    pub(super) generation: u64,
    pub(super) candidate_count: u32,
    pub(super) liveness_generation: u64,
    pub(super) manifest_digest: [u8; 32],
    pub(super) catalog_generation: u64,
    pub(super) catalog_digest: [u8; 32],
    pub(super) profile_identity: u32,
    pub(super) profile_version: u32,
    pub(super) profile_digest: [u8; 32],
    pub(super) catalog_successor_proof_digest: [u8; 32],
    pub(super) segment_pool_identity_digest: [u8; 32],
    pub(super) disposition_set_digest: [u8; 32],
    pub(super) reader_device: u64,
    pub(super) reader_mount: u64,
    pub(super) reader_file: u64,
    pub(super) candidate_set_digest: [u8; 32],
    pub(super) digest_offset: usize,
    pub(super) checksum_offset: usize,
}

pub(super) fn decode(encoded: &[u8]) -> Result<DecodedIntentHeader, Error> {
    require_minimum(encoded, format::HEADER_LENGTH)?;
    validate_fixed_fields(encoded)?;
    let candidate_count = read_u32(encoded, 44)?;
    let total_length = format::canonical_length(candidate_count).ok_or(Error::LengthOverflow)?;
    require_declared_length(encoded, total_length)?;
    require_exact(encoded, total_length)?;
    let checksum_offset = total_length.checked_sub(32).ok_or(Error::LengthOverflow)?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or(Error::LengthOverflow)?;
    Ok(DecodedIntentHeader {
        generation: read_u64(encoded, 32)?,
        candidate_count,
        liveness_generation: read_u64(encoded, 48)?,
        manifest_digest: read_array(encoded, 56)?,
        catalog_generation: read_u64(encoded, 88)?,
        catalog_digest: read_array(encoded, 96)?,
        profile_identity: read_u32(encoded, 128)?,
        profile_version: read_u32(encoded, 132)?,
        profile_digest: read_array(encoded, 136)?,
        catalog_successor_proof_digest: read_array(encoded, 168)?,
        segment_pool_identity_digest: read_array(encoded, 200)?,
        disposition_set_digest: read_array(encoded, 232)?,
        reader_device: read_u64(encoded, 264)?,
        reader_mount: read_u64(encoded, 272)?,
        reader_file: read_u64(encoded, 280)?,
        candidate_set_digest: read_array(encoded, format::CANDIDATE_SET_DIGEST_OFFSET)?,
        digest_offset,
        checksum_offset,
    })
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), Error> {
    let magic = read_array(encoded, 0)?;
    if magic != format::MAGIC {
        return Err(Error::InvalidMagic { observed: magic });
    }
    require_u16(encoded, 16, format::VERSION, |expected, observed| {
        Error::UnsupportedVersion { expected, observed }
    })?;
    require_u16(
        encoded,
        18,
        format::RECORD_HEADER_LENGTH,
        |expected, observed| Error::InvalidHeaderLength { expected, observed },
    )?;
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(Error::UnsupportedFlags { observed: flags });
    }
    require_u16(
        encoded,
        40,
        format::RECORD_CANDIDATE_WIDTH,
        |expected, observed| Error::InvalidCandidateWidth { expected, observed },
    )?;
    require_zero(encoded, 42, 2, "candidate")
}

fn require_declared_length(encoded: &[u8], total_length: usize) -> Result<(), Error> {
    let observed = read_u64(encoded, 24)?;
    let expected = u64::try_from(total_length).map_err(|_| Error::LengthOverflow)?;
    if observed == expected {
        Ok(())
    } else {
        Err(Error::DeclaredLengthMismatch { expected, observed })
    }
}
