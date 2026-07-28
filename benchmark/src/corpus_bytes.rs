//! Deterministic byte generators for benchmark corpus members.

use crate::CorpusError;

const TINY_COUNT: usize = 256;

pub(super) fn repeated_pattern(
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

pub(super) fn deterministic_binary(length: usize, seed: u64) -> Result<Vec<u8>, CorpusError> {
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

pub(super) fn reserved(length: usize, target: &'static str) -> Result<Vec<u8>, CorpusError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|source| CorpusError::Allocation { target, source })?;
    Ok(output)
}

pub(super) fn tiny_blobs() -> Result<Vec<Box<[u8]>>, CorpusError> {
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
