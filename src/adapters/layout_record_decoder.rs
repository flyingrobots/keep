//! Bounded parse, validate, and admit pipeline for flat layouts.

use super::layout_header_decoder::{DecodedHeader, decode_header};
use super::layout_record_format::{
    CHECKSUM_LENGTH, ENTRY_LENGTH, HEADER_LENGTH, PROFILE_HASH_ALGORITHM, PROFILE_IDENTITY_VERSION,
    calculate_layout_id, record_checksum,
};
use super::layout_record_framing::{ValidatedFraming, validate_framing};
use super::{LayoutDecodeError, LayoutDecodePolicy};
use crate::BlobId;
use crate::chunk::{ChunkId, ChunkLength, ChunkOffset};
use crate::layout::{AdmittedLayout, LayoutEntry};
use crate::profile::{RegisteredStorageProfile, StorageProfileId};

pub(super) fn decode_layout(
    encoded: &[u8],
    policy: LayoutDecodePolicy,
) -> Result<AdmittedLayout, LayoutDecodeError> {
    let header = decode_header(encoded)?;
    let framing = validate_framing(
        header.record_length,
        header.entry_count,
        encoded.len(),
        policy,
    )?;
    verify_checksum(encoded, &framing)?;
    let target = decode_target(&header)?;
    let profile = admit_profile(&header)?;
    let entries = decode_entries(encoded, &framing)?;
    let layout =
        AdmittedLayout::from_decoded_entries(target, profile, entries, policy.entry_limit())
            .map_err(|source| LayoutDecodeError::Validation { source })?;
    verify_expected_identity(encoded, framing.record_length, policy)?;
    Ok(layout)
}

fn verify_checksum(encoded: &[u8], framing: &ValidatedFraming) -> Result<(), LayoutDecodeError> {
    let Some((covered, checksum_bytes)) = encoded.split_at_checked(framing.checksum_start) else {
        return Err(LayoutDecodeError::TruncatedRecord {
            expected: framing.record_length.get(),
            observed: encoded.len(),
        });
    };
    let Some((observed, trailing)) = checksum_bytes.split_first_chunk::<32>() else {
        return Err(LayoutDecodeError::TruncatedRecord {
            expected: framing.record_length.get(),
            observed: encoded.len(),
        });
    };
    if !trailing.is_empty() {
        return Err(LayoutDecodeError::TrailingData {
            expected: framing.record_length.get(),
            observed: encoded.len(),
        });
    }
    let covered_length = framing
        .record_length
        .get()
        .checked_sub(CHECKSUM_LENGTH)
        .ok_or(LayoutDecodeError::RecordLengthArithmetic {
            entry_count: framing.entry_count,
        })?;
    let expected = record_checksum(covered, covered_length);
    if &expected == observed {
        return Ok(());
    }
    Err(LayoutDecodeError::ChecksumMismatch {
        expected,
        observed: *observed,
    })
}

fn decode_target(header: &DecodedHeader) -> Result<BlobId, LayoutDecodeError> {
    BlobId::parse_binary(&header.target_blob_id)
        .map_err(|source| LayoutDecodeError::BlobId { source })
}

fn admit_profile(header: &DecodedHeader) -> Result<RegisteredStorageProfile, LayoutDecodeError> {
    if header.profile_identity_version != PROFILE_IDENTITY_VERSION {
        return Err(LayoutDecodeError::UnsupportedStorageProfileVersion {
            expected: PROFILE_IDENTITY_VERSION,
            observed: header.profile_identity_version,
        });
    }
    if header.profile_hash_algorithm != PROFILE_HASH_ALGORITHM {
        return Err(LayoutDecodeError::UnsupportedStorageProfileAlgorithm {
            expected: PROFILE_HASH_ALGORITHM,
            observed: header.profile_hash_algorithm,
        });
    }
    let id = StorageProfileId::from_validated_digest(header.profile_digest);
    RegisteredStorageProfile::admit(id)
        .map_err(|source| LayoutDecodeError::StorageProfile { source })
}

fn decode_entries(
    encoded: &[u8],
    framing: &ValidatedFraming,
) -> Result<Vec<LayoutEntry>, LayoutDecodeError> {
    let header_width = usize::from(HEADER_LENGTH);
    let Some(covered) = encoded.get(..framing.checksum_start) else {
        return Err(truncated(encoded, framing));
    };
    let Some(entries_bytes) = covered.get(header_width..) else {
        return Err(truncated(encoded, framing));
    };
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(framing.entry_capacity)
        .map_err(|source| LayoutDecodeError::Allocation {
            requested: framing.entry_capacity,
            source,
        })?;
    let mut chunks = entries_bytes.chunks_exact(usize::from(ENTRY_LENGTH));
    let mut index = 0_u32;
    for entry_bytes in chunks.by_ref() {
        entries.push(decode_entry(entry_bytes, index, encoded, framing)?);
        index = index
            .checked_add(1)
            .ok_or(LayoutDecodeError::RecordLengthArithmetic {
                entry_count: framing.entry_count,
            })?;
    }
    if !chunks.remainder().is_empty() || index != framing.entry_count {
        return Err(LayoutDecodeError::RecordLengthArithmetic {
            entry_count: framing.entry_count,
        });
    }
    Ok(entries)
}

const fn decode_entry(
    encoded: &[u8],
    index: u32,
    complete: &[u8],
    framing: &ValidatedFraming,
) -> Result<LayoutEntry, LayoutDecodeError> {
    let Some((offset_bytes, remainder)) = encoded.split_first_chunk::<8>() else {
        return Err(truncated(complete, framing));
    };
    let Some((length_bytes, remainder)) = remainder.split_first_chunk::<4>() else {
        return Err(truncated(complete, framing));
    };
    let Some((digest, trailing)) = remainder.split_first_chunk::<32>() else {
        return Err(truncated(complete, framing));
    };
    if !trailing.is_empty() {
        return Err(LayoutDecodeError::RecordLengthArithmetic {
            entry_count: framing.entry_count,
        });
    }
    let raw_length = u32::from_be_bytes(*length_bytes);
    if raw_length == 0 {
        return Err(LayoutDecodeError::ZeroChunkLength { index });
    }
    let offset = ChunkOffset::from_validated(u64::from_be_bytes(*offset_bytes));
    let length = ChunkLength::from_validated(raw_length);
    let id = ChunkId::from_validated_parts(length, *digest);
    Ok(LayoutEntry::from_validated_parts(offset, id))
}

fn verify_expected_identity(
    encoded: &[u8],
    record_length: crate::LayoutRecordLength,
    policy: LayoutDecodePolicy,
) -> Result<(), LayoutDecodeError> {
    let Some(expected) = policy.expected_id() else {
        return Ok(());
    };
    calculate_layout_id(encoded, record_length)
        .verify_expected(expected)
        .map_err(|source| LayoutDecodeError::LayoutIdentity { source })
}

const fn truncated(encoded: &[u8], framing: &ValidatedFraming) -> LayoutDecodeError {
    LayoutDecodeError::TruncatedRecord {
        expected: framing.record_length.get(),
        observed: encoded.len(),
    }
}

impl AdmittedLayout {
    /// Decodes, validates, and admits one exact canonical version-1 record.
    ///
    /// The fixed header and immutable wire bounds are checked before the
    /// decoder allocates. It then materializes one entry collection bounded by
    /// [`LayoutDecodePolicy::entry_limit`]. This operation performs no I/O and
    /// proves structure and registered-profile admission, not content
    /// possession, natural CDC boundaries, or reconstruction.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutDecodeError`] at the first deterministic failed framing,
    /// checksum, nested-coordinate, resource, semantic, or expected-identity
    /// law.
    pub fn decode_record(
        encoded: &[u8],
        policy: LayoutDecodePolicy,
    ) -> Result<Self, LayoutDecodeError> {
        decode_layout(encoded, policy)
    }
}
