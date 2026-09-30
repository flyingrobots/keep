//! Parsing and application of the frozen segment-store mutation ledgers.

use std::io;

use crate::support::{decode_hex, field, invalid_corpus};

const V1_MUTATIONS: &str = include_str!("../../conformance/segment-store/v1/mutations.tsv");
const V2_MUTATIONS: &str = include_str!("../../conformance/segment-store/v2/mutations.tsv");

/// The format a ledger row belongs to.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Format {
    V1,
    V2,
}

/// One parsed immutable mutation-ledger row.
pub(crate) struct MutationCase {
    pub(crate) format: Format,
    pub(crate) case: &'static str,
    pub(crate) record: &'static str,
    pub(crate) base_fixture: &'static str,
    pub(crate) operation: &'static str,
    pub(crate) offset: usize,
    pub(crate) span_length: usize,
    pub(crate) parameter: &'static str,
    pub(crate) checksum_posture: &'static str,
    pub(crate) expected_outcome: &'static str,
    pub(crate) stage: &'static str,
    pub(crate) requirement: &'static str,
}

/// Parses every row of both ledgers.
///
/// # Errors
///
/// Returns a corpus error when a header, field, or integer is malformed.
pub(crate) fn mutation_cases() -> Result<Vec<MutationCase>, io::Error> {
    let mut cases = Vec::new();
    for (format, ledger, header) in [
        (Format::V1, V1_MUTATIONS, "keep.segment-store-mutations/v1"),
        (Format::V2, V2_MUTATIONS, "keep.segment-store-mutations/v2"),
    ] {
        let mut lines = ledger.lines();
        if lines.next() != Some(header) {
            return Err(invalid_corpus("mutation ledger header is not canonical"));
        }
        if lines.next()
            != Some(
                "case\trecord\tbase_fixture\toperation\toffset\tspan_length\tparameter\t\
                 checksum_posture\texpected_outcome\tstage\trequirement",
            )
        {
            return Err(invalid_corpus("mutation ledger columns are not canonical"));
        }
        for row in lines {
            cases.push(parse_case(format, row)?);
        }
    }
    Ok(cases)
}

fn parse_case(format: Format, row: &'static str) -> Result<MutationCase, io::Error> {
    Ok(MutationCase {
        format,
        case: field(row, 0)?,
        record: field(row, 1)?,
        base_fixture: field(row, 2)?,
        operation: field(row, 3)?,
        offset: parse_usize(field(row, 4)?, "invalid mutation offset")?,
        span_length: parse_usize(field(row, 5)?, "invalid mutation span length")?,
        parameter: field(row, 6)?,
        checksum_posture: field(row, 7)?,
        expected_outcome: field(row, 8)?,
        stage: field(row, 9)?,
        requirement: field(row, 10)?,
    })
}

fn parse_usize(text: &str, message: &'static str) -> Result<usize, io::Error> {
    text.parse().map_err(|_source| invalid_corpus(message))
}

impl MutationCase {
    /// The base fixture's exact bytes.
    ///
    /// # Errors
    ///
    /// Returns a corpus error when the fixture is unknown or malformed.
    pub(crate) fn base_bytes(&self) -> Result<Vec<u8>, io::Error> {
        let hex = super::fixtures::fixture(self.format, self.base_fixture)?;
        decode_hex(hex.strip_suffix('\n').unwrap_or(hex))
    }

    /// Applies the frozen operation to the base fixture, before any
    /// checksum posture.
    ///
    /// # Errors
    ///
    /// Returns a corpus error for an unknown operation, an out-of-bounds
    /// span, or a parameter of the wrong width.
    pub(crate) fn mutated_bytes(&self) -> Result<Vec<u8>, io::Error> {
        let mut bytes = self.base_bytes()?;
        match self.operation {
            "replace-v1" => {
                let parameter = decode_parameter(self.parameter)?;
                require_width(self, &parameter)?;
                span_mut(&mut bytes, self.offset, self.span_length)?.copy_from_slice(&parameter);
            }
            "xor-v1" => {
                let parameter = decode_parameter(self.parameter)?;
                require_width(self, &parameter)?;
                for (target, mask) in span_mut(&mut bytes, self.offset, self.span_length)?
                    .iter_mut()
                    .zip(&parameter)
                {
                    *target ^= mask;
                }
            }
            "truncate-v1" => {
                if self.offset > bytes.len() || self.span_length != 0 {
                    return Err(invalid_corpus("truncate mutation is out of bounds"));
                }
                bytes.truncate(self.offset);
            }
            "append-v1" => {
                if self.offset != bytes.len() || self.span_length != 0 {
                    return Err(invalid_corpus("append mutation must name the fixture end"));
                }
                bytes.extend_from_slice(&decode_parameter(self.parameter)?);
            }
            "delete-v1" => {
                let end = span_end(self.offset, self.span_length)?;
                if bytes.get(self.offset..end).is_none() || self.parameter != "-" {
                    return Err(invalid_corpus("delete mutation is out of bounds"));
                }
                drop(bytes.drain(self.offset..end));
            }
            _ => return Err(invalid_corpus("unknown segment-store mutation operation")),
        }
        Ok(bytes)
    }
}

fn decode_parameter(parameter: &str) -> Result<Vec<u8>, io::Error> {
    if parameter == "-" {
        return Ok(Vec::new());
    }
    decode_hex(parameter)
}

fn require_width(case: &MutationCase, parameter: &[u8]) -> Result<(), io::Error> {
    if parameter.len() == case.span_length {
        Ok(())
    } else {
        Err(invalid_corpus(
            "mutation parameter width disagrees with its span",
        ))
    }
}

fn span_end(offset: usize, length: usize) -> Result<usize, io::Error> {
    offset
        .checked_add(length)
        .ok_or_else(|| invalid_corpus("mutation span overflows"))
}

fn span_mut(bytes: &mut [u8], offset: usize, length: usize) -> Result<&mut [u8], io::Error> {
    let end = span_end(offset, length)?;
    bytes
        .get_mut(offset..end)
        .ok_or_else(|| invalid_corpus("mutation span is out of bounds"))
}
