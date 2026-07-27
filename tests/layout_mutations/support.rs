//! Independent application of the frozen flat-layout mutation protocol.

use std::io;

use crate::support::{decode_hex, invalid_corpus};

const MUTATIONS: &str = include_str!("../../conformance/layout/v1/mutations.tsv");
const CHECKSUM_DOMAIN: &[u8; 16] = b"KEEP:LAYOUT:SUM\0";

/// One parsed immutable mutation-ledger row.
pub struct MutationCase<'a> {
    case: &'a str,
    base_case: &'a str,
    operation: &'a str,
    offset: usize,
    span_length: usize,
    parameter: &'a str,
    checksum_posture: &'a str,
    decision_phase: &'a str,
    expected_outcome: &'a str,
}

impl MutationCase<'_> {
    /// Returns the stable mutation case name.
    #[must_use]
    pub const fn case(&self) -> &str {
        self.case
    }

    /// Returns the trust phase expected to refuse the mutation.
    #[must_use]
    pub const fn decision_phase(&self) -> &str {
        self.decision_phase
    }

    /// Returns the exact frozen refusal class.
    #[must_use]
    pub const fn expected_outcome(&self) -> &str {
        self.expected_outcome
    }

    /// Applies the frozen operation and checksum posture to its base fixture.
    ///
    /// # Errors
    ///
    /// Returns an I/O-shaped corpus error for malformed fields, out-of-bounds
    /// spans, unsupported operations, or inconsistent mutation widths.
    pub fn mutated_record(&self) -> Result<Vec<u8>, io::Error> {
        let mut bytes = decode_hex(record_fixture(self.base_case)?)?;
        match self.operation {
            "replace-v1" => replace(&mut bytes, self, &decode_parameter(self.parameter)?)?,
            "xor-v1" => xor(&mut bytes, self, &decode_parameter(self.parameter)?)?,
            "insert-v1" => insert(&mut bytes, self, &decode_parameter(self.parameter)?)?,
            "delete-v1" => delete(&mut bytes, self)?,
            "swap-v1" => swap(&mut bytes, self)?,
            _ => return Err(invalid_corpus("unknown layout mutation operation")),
        }
        match self.checksum_posture {
            "recompute-v1" => recompute_checksum(&mut bytes)?,
            "preserve-v1" => {}
            _ => return Err(invalid_corpus("unknown checksum posture")),
        }
        Ok(bytes)
    }
}

/// Parses every immutable mutation-ledger row.
///
/// # Errors
///
/// Returns an I/O-shaped corpus error when any row is missing a field or
/// carries a noncanonical host-width integer.
pub fn mutation_cases() -> Result<Vec<MutationCase<'static>>, io::Error> {
    MUTATIONS
        .lines()
        .skip(2)
        .map(parse_case)
        .collect::<Result<Vec<_>, _>>()
}

fn parse_case(row: &'static str) -> Result<MutationCase<'static>, io::Error> {
    Ok(MutationCase {
        case: field(row, 0)?,
        base_case: field(row, 1)?,
        operation: field(row, 2)?,
        offset: field(row, 3)?
            .parse()
            .map_err(|_source| invalid_corpus("invalid mutation offset"))?,
        span_length: field(row, 4)?
            .parse()
            .map_err(|_source| invalid_corpus("invalid mutation span length"))?,
        parameter: field(row, 5)?,
        checksum_posture: field(row, 6)?,
        decision_phase: field(row, 7)?,
        expected_outcome: field(row, 8)?,
    })
}

fn replace(
    bytes: &mut [u8],
    mutation: &MutationCase<'_>,
    parameter: &[u8],
) -> Result<(), io::Error> {
    require_parameter_width(mutation, parameter)?;
    let span = mutable_span(bytes, mutation.offset, mutation.span_length)?;
    span.copy_from_slice(parameter);
    Ok(())
}

fn xor(bytes: &mut [u8], mutation: &MutationCase<'_>, parameter: &[u8]) -> Result<(), io::Error> {
    require_parameter_width(mutation, parameter)?;
    let span = mutable_span(bytes, mutation.offset, mutation.span_length)?;
    for (target, mask) in span.iter_mut().zip(parameter) {
        *target ^= mask;
    }
    Ok(())
}

fn insert(
    bytes: &mut Vec<u8>,
    mutation: &MutationCase<'_>,
    parameter: &[u8],
) -> Result<(), io::Error> {
    if mutation.span_length != 0 {
        return Err(invalid_corpus("insert mutation has a nonzero span"));
    }
    if mutation.offset > bytes.len() {
        return Err(invalid_corpus("insert mutation offset is out of bounds"));
    }
    bytes
        .try_reserve(parameter.len())
        .map_err(|_source| invalid_corpus("mutation allocation failed"))?;
    let inserted = bytes.splice(mutation.offset..mutation.offset, parameter.iter().copied());
    drop(inserted);
    Ok(())
}

fn delete(bytes: &mut Vec<u8>, mutation: &MutationCase<'_>) -> Result<(), io::Error> {
    if mutation.parameter != "-" {
        return Err(invalid_corpus("delete mutation has a parameter"));
    }
    let end = span_end(mutation.offset, mutation.span_length)?;
    if bytes.get(mutation.offset..end).is_none() {
        return Err(invalid_corpus("delete mutation span is out of bounds"));
    }
    drop(bytes.drain(mutation.offset..end));
    Ok(())
}

fn swap(bytes: &mut [u8], mutation: &MutationCase<'_>) -> Result<(), io::Error> {
    let other_offset = mutation
        .parameter
        .parse::<usize>()
        .map_err(|_source| invalid_corpus("invalid swap offset"))?;
    let first = immutable_span(bytes, mutation.offset, mutation.span_length)?.to_vec();
    let second = immutable_span(bytes, other_offset, mutation.span_length)?.to_vec();
    mutable_span(bytes, mutation.offset, mutation.span_length)?.copy_from_slice(&second);
    mutable_span(bytes, other_offset, mutation.span_length)?.copy_from_slice(&first);
    Ok(())
}

fn recompute_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let checksum_start = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("record is shorter than its checksum"))?;
    let covered = bytes
        .get(..checksum_start)
        .ok_or_else(|| invalid_corpus("checksum coverage is out of bounds"))?;
    let covered_length = u64::try_from(covered.len())
        .map_err(|_source| invalid_corpus("checksum coverage exceeds u64"))?;
    let mut state = blake3::Hasher::new();
    state.update(CHECKSUM_DOMAIN);
    state.update(&1_u16.to_be_bytes());
    state.update(&[1_u8]);
    state.update(covered);
    state.update(&covered_length.to_be_bytes());
    let checksum = *state.finalize().as_bytes();
    let slot = bytes
        .get_mut(checksum_start..)
        .ok_or_else(|| invalid_corpus("checksum slot is out of bounds"))?;
    if slot.len() != checksum.len() {
        return Err(invalid_corpus("checksum slot has the wrong width"));
    }
    slot.copy_from_slice(&checksum);
    Ok(())
}

fn require_parameter_width(mutation: &MutationCase<'_>, parameter: &[u8]) -> Result<(), io::Error> {
    if parameter.len() == mutation.span_length {
        return Ok(());
    }
    Err(invalid_corpus("mutation parameter width mismatch"))
}

fn mutable_span(bytes: &mut [u8], offset: usize, length: usize) -> Result<&mut [u8], io::Error> {
    let end = span_end(offset, length)?;
    bytes
        .get_mut(offset..end)
        .ok_or_else(|| invalid_corpus("mutation span is out of bounds"))
}

fn immutable_span(bytes: &[u8], offset: usize, length: usize) -> Result<&[u8], io::Error> {
    let end = span_end(offset, length)?;
    bytes
        .get(offset..end)
        .ok_or_else(|| invalid_corpus("mutation span is out of bounds"))
}

fn span_end(offset: usize, length: usize) -> Result<usize, io::Error> {
    offset
        .checked_add(length)
        .ok_or_else(|| invalid_corpus("mutation span overflows"))
}

fn decode_parameter(parameter: &str) -> Result<Vec<u8>, io::Error> {
    if parameter == "-" {
        return Ok(Vec::new());
    }
    decode_hex(parameter)
}

fn record_fixture(case: &str) -> Result<&'static str, io::Error> {
    match case {
        "empty" => Ok(include_str!("../../conformance/layout/v1/empty.layout.hex").trim_end()),
        "one-zero" => {
            Ok(include_str!("../../conformance/layout/v1/one-zero.layout.hex").trim_end())
        }
        "max-plus-one-zeros" => Ok(include_str!(
            "../../conformance/layout/v1/max-plus-one-zeros.layout.hex"
        )
        .trim_end()),
        "zeros-long" => {
            Ok(include_str!("../../conformance/layout/v1/zeros-long.layout.hex").trim_end())
        }
        _ => Err(invalid_corpus("unknown layout base fixture")),
    }
}

fn field(row: &str, index: usize) -> Result<&str, io::Error> {
    row.split('\t')
        .nth(index)
        .ok_or_else(|| invalid_corpus("layout mutation row is missing a field"))
}
