//! Assembly rules for the deterministic benchmark corpus.

use crate::corpus_bytes::{deterministic_binary, repeated_pattern, tiny_blobs};
use crate::corpus_edits::{EDIT_LENGTH, delete, insert, substitute};
use crate::corpus_identity::{identify_members, member_names, total_bytes};
use crate::{BenchmarkCorpus, CorpusError};

const LARGE_BYTES: usize = 1_048_576;
const EDIT_BYTES: usize = 2_097_152;
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
