//! This boundary module owns integrity admission after interrupted record framing admits.

use super::{RetentionManifestDecodeError as ManifestError, RetentionRootDecodeError as RootError};

pub(super) fn root(encoded: &[u8]) -> Result<(), RootError> {
    if encoded.len() < super::root_header_decoder::HEADER_LENGTH {
        return Ok(());
    }
    let total = usize::try_from(super::root_field_decoder::read_u64(encoded, 24)?)
        .map_err(|_| RootError::LengthOverflow)?;
    let checksum_offset = total.checked_sub(32).ok_or(RootError::LengthOverflow)?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or(RootError::LengthOverflow)?;
    super::root_integrity::verify_prefix(encoded, digest_offset, checksum_offset)?;
    let namespace_length = usize::from(super::root_field_decoder::read_u16(encoded, 40)?);
    let body_start = super::root_header_decoder::HEADER_LENGTH
        .checked_add(namespace_length)
        .ok_or(RootError::LengthOverflow)?;
    if let Some(anchors) = encoded.get(body_start..digest_offset) {
        let _digest = super::root_integrity::verify_anchor_set(
            super::root_field_decoder::read_u32(encoded, 44)?,
            anchors,
            super::root_field_decoder::read_array(encoded, 148)?,
        )?;
    }
    if let Some(anchors) = encoded.get(body_start..encoded.len().min(digest_offset)) {
        super::root_anchor_decoder::admit_prefix(anchors, body_start)?;
    }
    Ok(())
}

pub(super) fn manifest(encoded: &[u8]) -> Result<(), ManifestError> {
    if encoded.len() < super::manifest_header_decoder::HEADER_LENGTH {
        return Ok(());
    }
    let total = usize::try_from(super::manifest_field_decoder::read_u64(encoded, 24)?)
        .map_err(|_| ManifestError::LengthOverflow)?;
    let checksum_offset = total.checked_sub(32).ok_or(ManifestError::LengthOverflow)?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or(ManifestError::LengthOverflow)?;
    super::manifest_integrity::verify_prefix(encoded, digest_offset, checksum_offset)?;
    if let Some(entries) = encoded.get(super::manifest_header_decoder::HEADER_LENGTH..digest_offset)
    {
        super::manifest_integrity::verify_entry_set(
            super::manifest_field_decoder::read_u32(encoded, 44)?,
            entries,
            super::manifest_field_decoder::read_array(encoded, 80)?,
        )?;
    }
    if let Some(entries) =
        encoded.get(super::manifest_header_decoder::HEADER_LENGTH..encoded.len().min(digest_offset))
    {
        super::manifest_entry_decoder::admit_prefix(entries)?;
    }
    Ok(())
}
