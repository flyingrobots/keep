//! Canonical identity fixture loading.

use std::collections::BTreeSet;

use keep::{BlobId, BlobIdTextParseError};

use super::harness_failure::HarnessFailure;

const IDENTITIES: &str = include_str!("../../conformance/golden-file-worldline/v1/identities.tsv");
const SMALL_TEXT: &[u8] =
    include_bytes!("../../conformance/golden-file-worldline/v1/inputs/small-text.txt");
const STATE_A: &[u8] =
    include_bytes!("../../conformance/golden-file-worldline/v1/inputs/state-a.txt");
const STATE_B: &[u8] =
    include_bytes!("../../conformance/golden-file-worldline/v1/inputs/state-b.txt");

const IDENTITY_SCHEMA: &str = "# keep.golden-file-worldline.identities/v1";
const IDENTITY_COLUMNS: &str = concat!(
    "case\tsource_kind\tsource_parameter\trepetitions\tlogical_length\t",
    "canonical_text\tcanonical_binary_hex"
);
const BYTE_RAMP_LENGTH: usize = 256;
const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_TOTAL_BYTES: usize = 1_048_911;

pub(super) struct IdentityCase {
    pub(super) name: &'static str,
    source: FixtureSource,
    logical_length: usize,
    canonical_text: &'static str,
    canonical_binary_hex: &'static str,
}

enum FixtureSource {
    Empty,
    File(&'static [u8]),
    ByteRamp { repetitions: usize },
}

impl IdentityCase {
    pub(super) fn bytes(&self) -> Result<Vec<u8>, HarnessFailure> {
        let bytes = match self.source {
            FixtureSource::Empty => Vec::new(),
            FixtureSource::File(bytes) => bytes.to_vec(),
            FixtureSource::ByteRamp { repetitions } => byte_ramp(repetitions)?,
        };
        if bytes.len() != self.logical_length {
            return Err(HarnessFailure::corpus("identity source length moved"));
        }
        Ok(bytes)
    }

    pub(super) fn expected_id(&self) -> Result<BlobId, BlobIdTextParseError> {
        self.canonical_text.parse()
    }

    pub(super) const fn expected_text(&self) -> &'static str {
        self.canonical_text
    }

    pub(super) fn expected_binary(&self) -> Result<Vec<u8>, HarnessFailure> {
        decode_hex(self.canonical_binary_hex)
    }
}

pub(super) fn identity_cases() -> Result<Vec<IdentityCase>, HarnessFailure> {
    let mut lines = IDENTITIES.lines();
    validate_header(lines.next(), IDENTITY_SCHEMA)?;
    validate_header(lines.next(), IDENTITY_COLUMNS)?;
    let mut cases = Vec::new();
    let mut seen = BTreeSet::new();
    let mut declared_total = 0_usize;
    for line in lines {
        if line.is_empty() {
            return Err(HarnessFailure::corpus("blank identity row"));
        }
        let case = parse_case(line)?;
        if !seen.insert(case.name) {
            return Err(HarnessFailure::corpus("duplicate identity case"));
        }
        if case.logical_length > MAX_SOURCE_BYTES {
            return Err(HarnessFailure::corpus("identity source exceeds bound"));
        }
        declared_total = declared_total
            .checked_add(case.logical_length)
            .ok_or_else(|| HarnessFailure::corpus("identity corpus length overflow"))?;
        if declared_total > MAX_TOTAL_BYTES {
            return Err(HarnessFailure::corpus("identity corpus exceeds bound"));
        }
        cases.push(case);
    }
    Ok(cases)
}

pub(super) fn find_case(name: &str) -> Result<IdentityCase, HarnessFailure> {
    identity_cases()?
        .into_iter()
        .find(|case| case.name == name)
        .ok_or_else(|| HarnessFailure::corpus("referenced identity case is absent"))
}

pub(super) fn decode_hex(encoded: &str) -> Result<Vec<u8>, HarnessFailure> {
    if !encoded.len().is_multiple_of(2) {
        return Err(HarnessFailure::corpus("fixture hex has odd length"));
    }
    let mut decoded = Vec::with_capacity(
        encoded
            .len()
            .checked_div(2)
            .ok_or_else(|| HarnessFailure::corpus("fixture hex length division failed"))?,
    );
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = pair
            .first()
            .copied()
            .ok_or_else(|| HarnessFailure::corpus("fixture hex pair is empty"))?;
        let low = pair
            .get(1)
            .copied()
            .ok_or_else(|| HarnessFailure::corpus("fixture hex pair is truncated"))?;
        let shifted = fixture_nibble(high)? << 4;
        decoded.push(shifted | fixture_nibble(low)?);
    }
    Ok(decoded)
}

fn parse_case(line: &'static str) -> Result<IdentityCase, HarnessFailure> {
    let mut fields = line.split('\t');
    let name = field(&mut fields)?;
    let kind = field(&mut fields)?;
    let parameter = field(&mut fields)?;
    let repetitions = decimal(field(&mut fields)?)?;
    let logical_length = decimal(field(&mut fields)?)?;
    let canonical_text = field(&mut fields)?;
    let canonical_binary_hex = field(&mut fields)?;
    if fields.next().is_some() {
        return Err(HarnessFailure::corpus("identity row has trailing fields"));
    }
    let source = parse_source(kind, parameter, repetitions)?;
    Ok(IdentityCase {
        name,
        source,
        logical_length,
        canonical_text,
        canonical_binary_hex,
    })
}

fn parse_source(
    kind: &str,
    parameter: &str,
    repetitions: usize,
) -> Result<FixtureSource, HarnessFailure> {
    match (kind, parameter, repetitions) {
        ("empty-v1", "-", 1) => Ok(FixtureSource::Empty),
        ("file-v1", "inputs/small-text.txt", 1) => Ok(FixtureSource::File(SMALL_TEXT)),
        ("file-v1", "inputs/state-a.txt", 1) => Ok(FixtureSource::File(STATE_A)),
        ("file-v1", "inputs/state-b.txt", 1) => Ok(FixtureSource::File(STATE_B)),
        ("byte-ramp-v1", "-", repetitions) => {
            let generated = repetitions
                .checked_mul(BYTE_RAMP_LENGTH)
                .ok_or_else(|| HarnessFailure::corpus("byte-ramp length overflow"))?;
            if generated > MAX_SOURCE_BYTES {
                return Err(HarnessFailure::corpus("byte-ramp exceeds source bound"));
            }
            Ok(FixtureSource::ByteRamp { repetitions })
        }
        _ => Err(HarnessFailure::corpus("unsupported fixture source")),
    }
}

fn byte_ramp(repetitions: usize) -> Result<Vec<u8>, HarnessFailure> {
    let capacity = repetitions
        .checked_mul(BYTE_RAMP_LENGTH)
        .ok_or_else(|| HarnessFailure::corpus("byte-ramp length overflow"))?;
    let mut bytes = Vec::with_capacity(capacity);
    for _ in 0..repetitions {
        bytes.extend(0_u8..=u8::MAX);
    }
    Ok(bytes)
}

fn field(fields: &mut std::str::Split<'static, char>) -> Result<&'static str, HarnessFailure> {
    fields
        .next()
        .ok_or_else(|| HarnessFailure::corpus("identity row is missing a field"))
}

fn decimal(value: &str) -> Result<usize, HarnessFailure> {
    if value != "0"
        && (value.is_empty()
            || value.starts_with('0')
            || !value.as_bytes().iter().all(u8::is_ascii_digit))
    {
        return Err(HarnessFailure::corpus("fixture decimal is noncanonical"));
    }
    value
        .parse::<usize>()
        .map_err(|_source| HarnessFailure::corpus("fixture decimal is invalid"))
}

fn validate_header(observed: Option<&str>, expected: &str) -> Result<(), HarnessFailure> {
    if observed == Some(expected) {
        Ok(())
    } else {
        Err(HarnessFailure::corpus("fixture header mismatch"))
    }
}

const fn fixture_nibble(value: u8) -> Result<u8, HarnessFailure> {
    match value {
        b'0' => Ok(0),
        b'1' => Ok(1),
        b'2' => Ok(2),
        b'3' => Ok(3),
        b'4' => Ok(4),
        b'5' => Ok(5),
        b'6' => Ok(6),
        b'7' => Ok(7),
        b'8' => Ok(8),
        b'9' => Ok(9),
        b'a' => Ok(10),
        b'b' => Ok(11),
        b'c' => Ok(12),
        b'd' => Ok(13),
        b'e' => Ok(14),
        b'f' => Ok(15),
        _ => Err(HarnessFailure::corpus(
            "fixture hex is not lowercase hexadecimal",
        )),
    }
}
