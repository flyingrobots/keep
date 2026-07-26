//! Deterministic primitive CDC source loading.

use std::collections::BTreeMap;

use super::fixture_syntax::{canonical_decimal, decode_hex, field, header};
use super::harness_failure::HarnessFailure;

const SOURCES: &str = include_str!("../../conformance/cdc-profile/v1/sources.tsv");
const SOURCE_BLOCK: &[u8] =
    include_bytes!("../../conformance/cdc-profile/v1/inputs/source-block.txt");
const SCHEMA: &str = "keep.cdc-sources/v1";
const COLUMNS: &str = "case\trecipe\tparameter\tcount\tlogical_length";
const EXPECTED_PRIMITIVE_CASES: usize = 20;
const MAXIMUM_SOURCE_BYTES: usize = 1_048_576;

pub(super) fn primitive_sources() -> Result<BTreeMap<&'static str, Vec<u8>>, HarnessFailure> {
    let mut lines = SOURCES.lines();
    header(lines.next(), SCHEMA)?;
    header(lines.next(), COLUMNS)?;
    let mut sources = BTreeMap::new();
    for line in lines {
        let (name, bytes) = parse_source(line)?;
        if sources.insert(name, bytes).is_some() {
            return Err(HarnessFailure::corpus("duplicate source case"));
        }
    }
    if sources.len() != EXPECTED_PRIMITIVE_CASES {
        return Err(HarnessFailure::corpus("primitive source count moved"));
    }
    Ok(sources)
}

fn parse_source(line: &'static str) -> Result<(&'static str, Vec<u8>), HarnessFailure> {
    let mut fields = line.split('\t');
    let name = field(&mut fields)?;
    let recipe = field(&mut fields)?;
    let parameter = field(&mut fields)?;
    let count = canonical_decimal(field(&mut fields)?)?;
    let declared_length = canonical_decimal(field(&mut fields)?)?;
    if fields.next().is_some() {
        return Err(HarnessFailure::corpus("source row has trailing fields"));
    }
    if declared_length > MAXIMUM_SOURCE_BYTES {
        return Err(HarnessFailure::corpus("source exceeds fixture bound"));
    }
    let bytes = source_bytes(recipe, parameter, count, declared_length)?;
    if bytes.len() != declared_length {
        return Err(HarnessFailure::corpus("source length moved"));
    }
    Ok((name, bytes))
}

fn source_bytes(
    recipe: &str,
    parameter: &str,
    count: usize,
    declared_length: usize,
) -> Result<Vec<u8>, HarnessFailure> {
    match recipe {
        "empty-v1" => {
            if parameter == "-" && count == 0 && declared_length == 0 {
                Ok(Vec::new())
            } else {
                Err(HarnessFailure::corpus("empty recipe is malformed"))
            }
        }
        "repeated-byte-v1" => repeated_pattern(parameter, count, 1),
        "alternating-v1" => repeated_pattern(parameter, count, 2),
        "file-repeat-v1" => file_repeat(parameter, count),
        "xorshift64-v1" => xorshift_bytes(parameter, count),
        _ => Err(HarnessFailure::corpus("unsupported source recipe")),
    }
}

fn repeated_pattern(
    encoded: &str,
    repetitions: usize,
    expected_width: usize,
) -> Result<Vec<u8>, HarnessFailure> {
    let pattern = decode_hex(encoded)?;
    if pattern.len() != expected_width {
        return Err(HarnessFailure::corpus("source pattern width moved"));
    }
    let length = pattern
        .len()
        .checked_mul(repetitions)
        .ok_or_else(|| HarnessFailure::corpus("source repetition overflow"))?;
    if length > MAXIMUM_SOURCE_BYTES {
        return Err(HarnessFailure::corpus("source repetition exceeds bound"));
    }
    Ok(pattern.repeat(repetitions))
}

fn file_repeat(path: &str, repetitions: usize) -> Result<Vec<u8>, HarnessFailure> {
    if path != "inputs/source-block.txt" {
        return Err(HarnessFailure::corpus("source file path is unsupported"));
    }
    let length = SOURCE_BLOCK
        .len()
        .checked_mul(repetitions)
        .ok_or_else(|| HarnessFailure::corpus("source file repetition overflow"))?;
    if length > MAXIMUM_SOURCE_BYTES {
        return Err(HarnessFailure::corpus(
            "source file repetition exceeds bound",
        ));
    }
    Ok(SOURCE_BLOCK.repeat(repetitions))
}

fn xorshift_bytes(seed_hex: &str, count: usize) -> Result<Vec<u8>, HarnessFailure> {
    if count > MAXIMUM_SOURCE_BYTES {
        return Err(HarnessFailure::corpus("xorshift source exceeds bound"));
    }
    let seed_bytes: [u8; 8] = decode_hex(seed_hex)?
        .try_into()
        .map_err(|_source| HarnessFailure::corpus("xorshift seed width moved"))?;
    let mut state = u64::from_be_bytes(seed_bytes);
    if state == 0 {
        return Err(HarnessFailure::corpus("xorshift seed is zero"));
    }
    let mut bytes = Vec::with_capacity(count);
    for _ in 0..count {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        bytes.push(
            u8::try_from(state & u64::from(u8::MAX))
                .map_err(|_source| HarnessFailure::corpus("xorshift byte escaped u8"))?,
        );
    }
    Ok(bytes)
}
