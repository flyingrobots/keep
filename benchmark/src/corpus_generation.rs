//! Generation rules for deterministic benchmark input bytes.

use keep::BlobId;

use crate::{BenchmarkCorpus, CorpusError};

const LARGE_BYTES: usize = 1_048_576;
const EDIT_BYTES: usize = 2_097_152;
const EDIT_OFFSET: usize = 4_096;
const EDIT_LENGTH: usize = 4_096;
const NEIGHBOR_OFFSET: usize = 8_192;
const TINY_COUNT: usize = 256;
const TEXT_VARIATION_LENGTH: usize = 16;
const TEXT_VARIATION_OFFSET: usize = 12;
const TEXT_PATTERN: &[u8] = b"pub fn keep_aaaaaaaaaaaaaaaa() { assert!(identity_is_stable); }\n";

impl BenchmarkCorpus {
    /// Generates the fixed, bounded, license-safe benchmark corpus.
    ///
    /// # Errors
    ///
    /// Returns [`CorpusError`] for bounded allocation, identity, coordinate,
    /// aggregate-overflow, or aggregate-limit failures.
    pub fn generate() -> Result<Self, CorpusError> {
        let large_text = source_text(LARGE_BYTES, "large-text")?;
        let large_binary = deterministic_binary(LARGE_BYTES, 0x0123_4567_89ab_cdef)?;
        let edit_base = source_text(EDIT_BYTES, "edit-base")?;
        let insertion = repeated_pattern(EDIT_LENGTH, b"KEEP-INSERT\n", "insertion")?;
        let early_insertion = insert(&edit_base, &insertion)?;
        let early_deletion = delete(&edit_base)?;
        let near_neighbor = substitute(&edit_base)?;
        let zero_dedup = deterministic_binary(EDIT_BYTES, 0xfedc_ba98_7654_3210)?;
        let tiny_blobs = tiny_blobs()?;
        let members = [
            large_text.as_slice(),
            large_binary.as_slice(),
            edit_base.as_slice(),
            early_insertion.as_slice(),
            early_deletion.as_slice(),
            near_neighbor.as_slice(),
            zero_dedup.as_slice(),
        ];
        let names = member_names();
        let identities = identify_members(members, names)?;
        let total_bytes = total_bytes(members, &tiny_blobs)?;
        if total_bytes > Self::TOTAL_BYTE_LIMIT {
            return Err(CorpusError::TotalByteLimitExceeded {
                limit: Self::TOTAL_BYTE_LIMIT,
                observed: total_bytes,
            });
        }
        Ok(Self {
            large_text,
            large_binary,
            edit_base,
            early_insertion,
            early_deletion,
            near_neighbor,
            zero_dedup,
            tiny_blobs,
            identities,
            total_bytes,
        })
    }
}

fn source_text(length: usize, target: &'static str) -> Result<Vec<u8>, CorpusError> {
    let mut output = repeated_pattern(length, TEXT_PATTERN, target)?;
    let mut cursor = TEXT_VARIATION_OFFSET;
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    while cursor < output.len() {
        let end = cursor
            .checked_add(TEXT_VARIATION_LENGTH)
            .ok_or(CorpusError::TotalLengthOverflow)?;
        let slots = output
            .get_mut(cursor..end)
            .ok_or(CorpusError::InvalidGeneratedRange { target })?;
        for slot in slots {
            state ^= state.wrapping_shl(13);
            state ^= state.wrapping_shr(7);
            state ^= state.wrapping_shl(17);
            let [variation, ..] = state.to_le_bytes();
            *slot = b'a'
                .checked_add(variation & 0x0f)
                .ok_or(CorpusError::TotalLengthOverflow)?;
        }
        cursor =
            cursor
                .checked_add(TEXT_PATTERN.len())
                .ok_or(CorpusError::InvalidGeneratedRange {
                    target: "source-text-variation",
                })?;
    }
    Ok(output)
}

fn repeated_pattern(
    length: usize,
    pattern: &[u8],
    target: &'static str,
) -> Result<Vec<u8>, CorpusError> {
    let mut output = reserved(length, target)?;
    while output.len() < length {
        let remaining = length
            .checked_sub(output.len())
            .ok_or(CorpusError::TotalLengthOverflow)?;
        let accepted = remaining.min(pattern.len());
        let bytes = pattern
            .get(..accepted)
            .ok_or(CorpusError::InvalidGeneratedRange { target })?;
        output.extend_from_slice(bytes);
    }
    Ok(output)
}

fn deterministic_binary(length: usize, seed: u64) -> Result<Vec<u8>, CorpusError> {
    let mut state = seed;
    let mut output = reserved(length, "deterministic-binary")?;
    for _index in 0..length {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        let [byte, ..] = state.to_le_bytes();
        output.push(byte);
    }
    Ok(output)
}

fn reserved(length: usize, target: &'static str) -> Result<Vec<u8>, CorpusError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|source| CorpusError::Allocation { target, source })?;
    Ok(output)
}

fn insert(base: &[u8], inserted: &[u8]) -> Result<Vec<u8>, CorpusError> {
    let length = base
        .len()
        .checked_add(inserted.len())
        .ok_or(CorpusError::TotalLengthOverflow)?;
    let mut output = reserved(length, "early-insertion")?;
    output.extend_from_slice(range(base, ..EDIT_OFFSET, "early-insertion-prefix")?);
    output.extend_from_slice(inserted);
    output.extend_from_slice(range(base, EDIT_OFFSET.., "early-insertion-suffix")?);
    Ok(output)
}

fn delete(base: &[u8]) -> Result<Vec<u8>, CorpusError> {
    let end = EDIT_OFFSET
        .checked_add(EDIT_LENGTH)
        .ok_or(CorpusError::TotalLengthOverflow)?;
    let length = base
        .len()
        .checked_sub(EDIT_LENGTH)
        .ok_or(CorpusError::InvalidGeneratedRange {
            target: "early-deletion",
        })?;
    let mut output = reserved(length, "early-deletion")?;
    output.extend_from_slice(range(base, ..EDIT_OFFSET, "early-deletion-prefix")?);
    output.extend_from_slice(range(base, end.., "early-deletion-suffix")?);
    Ok(output)
}

fn substitute(base: &[u8]) -> Result<Vec<u8>, CorpusError> {
    let mut output = reserved(base.len(), "near-neighbor")?;
    output.extend_from_slice(base);
    let end = NEIGHBOR_OFFSET
        .checked_add(EDIT_LENGTH)
        .ok_or(CorpusError::TotalLengthOverflow)?;
    let changed =
        output
            .get_mut(NEIGHBOR_OFFSET..end)
            .ok_or(CorpusError::InvalidGeneratedRange {
                target: "near-neighbor",
            })?;
    for byte in changed {
        *byte ^= 0xa5;
    }
    Ok(output)
}

fn tiny_blobs() -> Result<Vec<Box<[u8]>>, CorpusError> {
    let mut blobs = Vec::new();
    blobs
        .try_reserve_exact(TINY_COUNT)
        .map_err(|source| CorpusError::Allocation {
            target: "tiny-blob-index",
            source,
        })?;
    let mut seed = 0x6a09_e667_f3bc_c908_u64;
    for length in 1..=TINY_COUNT {
        blobs.push(deterministic_binary(length, seed)?.into_boxed_slice());
        seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    }
    Ok(blobs)
}

const fn member_names() -> [&'static str; BenchmarkCorpus::MEMBER_COUNT] {
    [
        "large-text",
        "large-binary",
        "edit-base",
        "early-insertion",
        "early-deletion",
        "near-neighbor",
        "zero-dedup",
    ]
}

fn identify_members(
    members: [&[u8]; BenchmarkCorpus::MEMBER_COUNT],
    names: [&'static str; BenchmarkCorpus::MEMBER_COUNT],
) -> Result<[BlobId; BenchmarkCorpus::MEMBER_COUNT], CorpusError> {
    let mut identities = [BlobId::hash_bytes(&[]).map_err(|source| CorpusError::Identity {
        member: "empty-initializer",
        source,
    })?; BenchmarkCorpus::MEMBER_COUNT];
    for ((identity, member), name) in identities.iter_mut().zip(members).zip(names) {
        *identity = BlobId::hash_bytes(member).map_err(|source| CorpusError::Identity {
            member: name,
            source,
        })?;
    }
    Ok(identities)
}

fn total_bytes(
    members: [&[u8]; BenchmarkCorpus::MEMBER_COUNT],
    tiny_blobs: &[Box<[u8]>],
) -> Result<usize, CorpusError> {
    members
        .into_iter()
        .map(<[u8]>::len)
        .chain(tiny_blobs.iter().map(|blob| blob.len()))
        .try_fold(0_usize, |total, length| {
            total
                .checked_add(length)
                .ok_or(CorpusError::TotalLengthOverflow)
        })
}

fn range<'a, R>(source: &'a [u8], range: R, target: &'static str) -> Result<&'a [u8], CorpusError>
where
    R: std::slice::SliceIndex<[u8], Output = [u8]>,
{
    source
        .get(range)
        .ok_or(CorpusError::InvalidGeneratedRange { target })
}
