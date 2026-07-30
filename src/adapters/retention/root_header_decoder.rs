//! This boundary module owns retention root header framing admission.

use super::RetentionRootDecodeError;
use super::root_field_decoder::{
    read_array, read_u16, read_u32, read_u64, require_exact, require_minimum, require_u16,
    require_zero,
};

pub(super) const HEADER_LENGTH: usize = 192;
const ANCHOR_WIDTH: usize = 119;
const TRAILER_LENGTH: usize = 64;

pub(super) struct DecodedRootHeader {
    pub(super) generation: u64,
    pub(super) namespace_length: usize,
    pub(super) anchor_count: u32,
    pub(super) profile_identity: u32,
    pub(super) profile_version: u32,
    pub(super) profile_digest: [u8; 32],
    pub(super) closure_nodes: u64,
    pub(super) closure_depth: u16,
    pub(super) closure_encoded_bytes: u64,
    pub(super) closure_physical_bytes: u64,
    pub(super) predecessor: [u8; 32],
    pub(super) anchor_set_digest: [u8; 32],
    pub(super) digest_offset: usize,
    pub(super) checksum_offset: usize,
}

pub(super) fn decode(encoded: &[u8]) -> Result<DecodedRootHeader, RetentionRootDecodeError> {
    require_minimum(encoded, HEADER_LENGTH)?;
    validate_fixed_fields(encoded)?;
    let namespace_length = usize::from(read_u16(encoded, 40)?);
    let anchor_count = read_u32(encoded, 44)?;
    let total_length = canonical_length(namespace_length, anchor_count)?;
    require_declared_length(encoded, total_length)?;
    require_exact(encoded, total_length)?;
    let checksum_offset = total_length
        .checked_sub(32)
        .ok_or(RetentionRootDecodeError::LengthOverflow)?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or(RetentionRootDecodeError::LengthOverflow)?;
    Ok(DecodedRootHeader {
        generation: read_u64(encoded, 32)?,
        namespace_length,
        anchor_count,
        profile_identity: read_u32(encoded, 48)?,
        profile_version: read_u32(encoded, 52)?,
        profile_digest: read_array(encoded, 56)?,
        closure_nodes: read_u64(encoded, 88)?,
        closure_depth: read_u16(encoded, 96)?,
        closure_encoded_bytes: read_u64(encoded, 100)?,
        closure_physical_bytes: read_u64(encoded, 108)?,
        predecessor: read_array(encoded, 116)?,
        anchor_set_digest: read_array(encoded, 148)?,
        digest_offset,
        checksum_offset,
    })
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    let magic = read_array(encoded, 0)?;
    if magic != *b"KEEP:RET:ROOT2\0\0" {
        return Err(RetentionRootDecodeError::InvalidMagic { observed: magic });
    }
    require_u16(encoded, 16, 2, |expected, observed| {
        RetentionRootDecodeError::UnsupportedVersion { expected, observed }
    })?;
    require_u16(encoded, 18, 192, |expected, observed| {
        RetentionRootDecodeError::InvalidHeaderLength { expected, observed }
    })?;
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(RetentionRootDecodeError::UnsupportedFlags { observed: flags });
    }
    require_u16(encoded, 42, 119, |expected, observed| {
        RetentionRootDecodeError::InvalidAnchorWidth { expected, observed }
    })?;
    require_zero(encoded, 98, 2, "limit")?;
    require_zero(encoded, 180, 12, "trailing header")
}

fn canonical_length(
    namespace_length: usize,
    anchor_count: u32,
) -> Result<usize, RetentionRootDecodeError> {
    let anchors = usize::try_from(anchor_count)
        .map_err(|_| RetentionRootDecodeError::LengthOverflow)?
        .checked_mul(ANCHOR_WIDTH)
        .ok_or(RetentionRootDecodeError::LengthOverflow)?;
    HEADER_LENGTH
        .checked_add(namespace_length)
        .and_then(|length| length.checked_add(anchors))
        .and_then(|length| length.checked_add(TRAILER_LENGTH))
        .ok_or(RetentionRootDecodeError::LengthOverflow)
}

fn require_declared_length(
    encoded: &[u8],
    total_length: usize,
) -> Result<(), RetentionRootDecodeError> {
    let observed = read_u64(encoded, 24)?;
    let expected =
        u64::try_from(total_length).map_err(|_| RetentionRootDecodeError::LengthOverflow)?;
    if observed == expected {
        Ok(())
    } else {
        Err(RetentionRootDecodeError::DeclaredLengthMismatch { expected, observed })
    }
}
