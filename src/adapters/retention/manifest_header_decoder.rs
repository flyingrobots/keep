//! This boundary module owns retention manifest header framing admission.

use super::RetentionManifestDecodeError;
use super::manifest_field_decoder::{
    read_array, read_u32, read_u64, require_exact, require_minimum, require_u16, require_zero,
};

pub(super) const HEADER_LENGTH: usize = 160;
const ENTRY_WIDTH: usize = 72;
const TRAILER_LENGTH: usize = 64;

pub(super) struct DecodedManifestHeader {
    pub(super) generation: u64,
    pub(super) entry_count: u32,
    pub(super) predecessor: [u8; 32],
    pub(super) entry_set_digest: [u8; 32],
    pub(super) digest_offset: usize,
    pub(super) checksum_offset: usize,
}

pub(super) fn decode(
    encoded: &[u8],
) -> Result<DecodedManifestHeader, RetentionManifestDecodeError> {
    require_minimum(encoded, HEADER_LENGTH)?;
    validate_fixed_fields(encoded)?;
    let entry_count = read_u32(encoded, 44)?;
    let total_length = canonical_length(entry_count)?;
    require_declared_length(encoded, total_length)?;
    require_exact(encoded, total_length)?;
    let checksum_offset = total_length
        .checked_sub(32)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    Ok(DecodedManifestHeader {
        generation: read_u64(encoded, 32)?,
        entry_count,
        predecessor: read_array(encoded, 48)?,
        entry_set_digest: read_array(encoded, 80)?,
        digest_offset,
        checksum_offset,
    })
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), RetentionManifestDecodeError> {
    let magic = read_array(encoded, 0)?;
    if magic != *b"KEEP:RET:LIVE2\0\0" {
        return Err(RetentionManifestDecodeError::InvalidMagic { observed: magic });
    }
    require_u16(encoded, 16, 2, |expected, observed| {
        RetentionManifestDecodeError::UnsupportedVersion { expected, observed }
    })?;
    require_u16(encoded, 18, 160, |expected, observed| {
        RetentionManifestDecodeError::InvalidHeaderLength { expected, observed }
    })?;
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(RetentionManifestDecodeError::UnsupportedFlags { observed: flags });
    }
    require_u16(encoded, 40, 72, |expected, observed| {
        RetentionManifestDecodeError::InvalidEntryWidth { expected, observed }
    })?;
    require_zero(encoded, 42, 2, "entry")?;
    require_zero(encoded, 112, 48, "trailing header")
}

fn canonical_length(entry_count: u32) -> Result<usize, RetentionManifestDecodeError> {
    let entries = usize::try_from(entry_count)
        .map_err(|_| RetentionManifestDecodeError::LengthOverflow)?
        .checked_mul(ENTRY_WIDTH)
        .ok_or(RetentionManifestDecodeError::LengthOverflow)?;
    HEADER_LENGTH
        .checked_add(entries)
        .and_then(|length| length.checked_add(TRAILER_LENGTH))
        .ok_or(RetentionManifestDecodeError::LengthOverflow)
}

fn require_declared_length(
    encoded: &[u8],
    total_length: usize,
) -> Result<(), RetentionManifestDecodeError> {
    let observed = read_u64(encoded, 24)?;
    let expected =
        u64::try_from(total_length).map_err(|_| RetentionManifestDecodeError::LengthOverflow)?;
    if observed == expected {
        Ok(())
    } else {
        Err(RetentionManifestDecodeError::DeclaredLengthMismatch { expected, observed })
    }
}
