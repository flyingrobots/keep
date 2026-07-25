//! Golden boundary fixture loading.

use std::collections::BTreeMap;

use super::harness_failure::HarnessFailure;
use super::source_corpus::canonical_decimal;

const BOUNDARIES: &str = include_str!("../../conformance/cdc-profile/v1/boundaries.tsv");
const SCHEMA: &str = "keep.cdc-boundaries/v1";
const COLUMNS: &str = "case\tchunk_count\tboundaries";
const EXPECTED_CASES: usize = 24;
const MAXIMUM_BOUNDARIES_PER_CASE: usize = 32;

pub(super) fn expected_boundaries() -> Result<BTreeMap<&'static str, Vec<u64>>, HarnessFailure> {
    let mut lines = BOUNDARIES.lines();
    header(lines.next(), SCHEMA)?;
    header(lines.next(), COLUMNS)?;
    let mut cases = BTreeMap::new();
    for line in lines {
        let (name, boundaries) = parse_case(line)?;
        if cases.insert(name, boundaries).is_some() {
            return Err(HarnessFailure::corpus("duplicate boundary case"));
        }
    }
    if cases.len() != EXPECTED_CASES {
        return Err(HarnessFailure::corpus("boundary case count moved"));
    }
    Ok(cases)
}

fn parse_case(line: &'static str) -> Result<(&'static str, Vec<u64>), HarnessFailure> {
    let mut fields = line.split('\t');
    let name = field(&mut fields)?;
    let count = canonical_decimal(field(&mut fields)?)?;
    let encoded = field(&mut fields)?;
    if fields.next().is_some() {
        return Err(HarnessFailure::corpus("boundary row has trailing fields"));
    }
    let boundaries = if encoded == "-" {
        Vec::new()
    } else {
        encoded
            .split(',')
            .map(parse_boundary)
            .collect::<Result<Vec<_>, _>>()?
    };
    if boundaries.len() != count || boundaries.len() > MAXIMUM_BOUNDARIES_PER_CASE {
        return Err(HarnessFailure::corpus(
            "boundary count is outside its declaration",
        ));
    }
    if boundaries
        .windows(2)
        .any(|pair| pair.first() >= pair.get(1))
    {
        return Err(HarnessFailure::corpus(
            "boundaries are not strictly increasing",
        ));
    }
    Ok((name, boundaries))
}

fn parse_boundary(value: &str) -> Result<u64, HarnessFailure> {
    canonical_decimal(value)?;
    value
        .parse::<u64>()
        .map_err(|_source| HarnessFailure::corpus("boundary does not fit u64"))
}

fn field(fields: &mut std::str::Split<'static, char>) -> Result<&'static str, HarnessFailure> {
    fields
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| HarnessFailure::corpus("boundary row is missing a field"))
}

fn header(observed: Option<&str>, expected: &str) -> Result<(), HarnessFailure> {
    if observed == Some(expected) {
        Ok(())
    } else {
        Err(HarnessFailure::corpus("boundary header mismatch"))
    }
}
