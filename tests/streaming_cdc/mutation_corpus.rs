//! Deterministic CDC mutation fixture application.

use std::collections::BTreeMap;

use super::fixture_syntax::{canonical_decimal, decode_hex, field, header};
use super::harness_failure::HarnessFailure;

const MUTATIONS: &str = include_str!("../../conformance/cdc-profile/v1/mutations.tsv");
const SCHEMA: &str = "keep.cdc-mutations/v1";
const COLUMNS: &str = "case\tbase_case\toperation\toffset\tspan_length\tvalue_hex\tlogical_length";
const EXPECTED_ALL_CASES: usize = 24;
const EXPECTED_TOTAL_BYTES: usize = 8_681_103;

pub(super) fn add_mutations(
    sources: &mut BTreeMap<&'static str, Vec<u8>>,
) -> Result<(), HarnessFailure> {
    let mut lines = MUTATIONS.lines();
    header(lines.next(), SCHEMA)?;
    header(lines.next(), COLUMNS)?;
    for line in lines {
        let mutation = parse_mutation(line)?;
        let base = sources
            .get(mutation.base_case)
            .ok_or_else(|| HarnessFailure::corpus("mutation base is absent"))?;
        let bytes = apply(base, &mutation)?;
        if bytes.len() != mutation.logical_length {
            return Err(HarnessFailure::corpus("mutation length moved"));
        }
        if sources.insert(mutation.name, bytes).is_some() {
            return Err(HarnessFailure::corpus("duplicate mutation case"));
        }
    }
    let total = sources.values().try_fold(0_usize, |accumulated, bytes| {
        accumulated
            .checked_add(bytes.len())
            .ok_or_else(|| HarnessFailure::corpus("CDC corpus total overflow"))
    })?;
    if sources.len() != EXPECTED_ALL_CASES || total != EXPECTED_TOTAL_BYTES {
        return Err(HarnessFailure::corpus("CDC corpus aggregate moved"));
    }
    Ok(())
}

struct Mutation {
    name: &'static str,
    base_case: &'static str,
    operation: &'static str,
    offset: usize,
    span_length: usize,
    value: Vec<u8>,
    logical_length: usize,
}

fn parse_mutation(line: &'static str) -> Result<Mutation, HarnessFailure> {
    let mut fields = line.split('\t');
    let name = field(&mut fields)?;
    let base_case = field(&mut fields)?;
    let operation = field(&mut fields)?;
    let offset = canonical_decimal(field(&mut fields)?)?;
    let span_length = canonical_decimal(field(&mut fields)?)?;
    let value_field = field(&mut fields)?;
    let value = if value_field == "-" {
        Vec::new()
    } else {
        decode_hex(value_field)?
    };
    let logical_length = canonical_decimal(field(&mut fields)?)?;
    if fields.next().is_some() {
        return Err(HarnessFailure::corpus("mutation row has trailing fields"));
    }
    Ok(Mutation {
        name,
        base_case,
        operation,
        offset,
        span_length,
        value,
        logical_length,
    })
}

fn apply(base: &[u8], mutation: &Mutation) -> Result<Vec<u8>, HarnessFailure> {
    let mut bytes = base.to_vec();
    let end = mutation
        .offset
        .checked_add(mutation.span_length)
        .ok_or_else(|| HarnessFailure::corpus("mutation span overflow"))?;
    if end > bytes.len() {
        return Err(HarnessFailure::corpus("mutation span escaped source"));
    }
    match mutation.operation {
        "insert-v1" => {
            if mutation.span_length != 0 || mutation.value.is_empty() {
                return Err(HarnessFailure::corpus("insert mutation is malformed"));
            }
            bytes.splice(
                mutation.offset..mutation.offset,
                mutation.value.iter().copied(),
            );
        }
        "delete-v1" => {
            if mutation.span_length == 0 || !mutation.value.is_empty() {
                return Err(HarnessFailure::corpus("delete mutation is malformed"));
            }
            bytes.drain(mutation.offset..end);
        }
        "xor-v1" => xor_span(&mut bytes, mutation, end)?,
        _ => return Err(HarnessFailure::corpus("unsupported mutation recipe")),
    }
    Ok(bytes)
}

fn xor_span(bytes: &mut [u8], mutation: &Mutation, end: usize) -> Result<(), HarnessFailure> {
    if mutation.span_length == 0 || mutation.value.len() != mutation.span_length {
        return Err(HarnessFailure::corpus("xor mutation is malformed"));
    }
    let target = bytes
        .get_mut(mutation.offset..end)
        .ok_or_else(|| HarnessFailure::corpus("xor mutation escaped source"))?;
    for (byte, mask) in target.iter_mut().zip(&mutation.value) {
        *byte ^= *mask;
    }
    Ok(())
}
