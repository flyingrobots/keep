//! This module owns independent identity materialization, hashing, and vectors.

use std::collections::{BTreeMap, BTreeSet};

use super::canonical_value::{EmptyHex, case_name, decimal, decoded_hex, unique};
use super::corpus_protocol::{MAX_SOURCE_BYTES, TableRow};
use super::{Corpus, GoldenError};

pub(super) const ALGORITHM: u8 = 1;
pub(super) const BINARY_LENGTH: usize = 59;
pub(super) const ID_MAGIC: [u8; 16] = *b"KEEP:BLOB:ID\0\0\0\0";
pub(super) const VERSION: u16 = 1;

const DATA_MAGIC: [u8; 16] = *b"KEEP:BLOB:DATA\0\0";
const MAX_TOTAL_BYTES: u64 = 1_048_911;
const IDENTITY_COLUMNS: [&str; 7] = [
    "case",
    "source_kind",
    "source_parameter",
    "repetitions",
    "logical_length",
    "canonical_text",
    "canonical_binary_hex",
];

pub(super) struct IdentityFixture {
    pub(super) content: Vec<u8>,
    pub(super) canonical_text: String,
    pub(super) canonical_binary: Vec<u8>,
}

pub(super) fn check_identities(
    corpus: &Corpus,
) -> Result<BTreeMap<String, IdentityFixture>, GoldenError> {
    let rows = corpus.rows(
        "identities.tsv",
        "# keep.golden-file-worldline.identities/v1",
        &IDENTITY_COLUMNS,
    )?;
    let mut fixtures = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut total = 0_u64;
    for row in rows {
        let name = case_name(row.field("case")?, "identities.tsv")?.to_owned();
        unique(&name, &mut seen, "identities.tsv")?;
        let (fixture, length) = identity_fixture(corpus, &row, &name)?;
        total = total
            .checked_add(length)
            .ok_or_else(|| GoldenError::violation("total materialized corpus overflow"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(GoldenError::violation(
                "total materialized corpus exceeds bound",
            ));
        }
        if fixtures.insert(name.clone(), fixture).is_some() {
            return Err(GoldenError::violation(format!(
                "identities.tsv: admitted identifier was displaced {name:?}"
            )));
        }
    }
    check_required_identities(&fixtures)?;
    check_state_insertion(&fixtures)?;
    Ok(fixtures)
}

fn identity_fixture(
    corpus: &Corpus,
    row: &TableRow,
    name: &str,
) -> Result<(IdentityFixture, u64), GoldenError> {
    let repetitions = source_number(row, "repetitions", name)?;
    let length = source_number(row, "logical_length", name)?;
    let content = source_bytes(corpus, row, repetitions, length)?;
    let identity_digest = digest(&content)?;
    let canonical_text = expected_text(length, &identity_digest);
    if row.field("canonical_text")? != canonical_text {
        return Err(GoldenError::violation(format!(
            "{name}: canonical text mismatch"
        )));
    }
    let binary = decoded_hex(
        row.field("canonical_binary_hex")?,
        &format!("{name} binary"),
        BINARY_LENGTH,
        EmptyHex::Refuse,
    )?;
    if binary != expected_binary(length, &identity_digest) {
        return Err(GoldenError::violation(format!(
            "{name}: canonical binary mismatch"
        )));
    }
    Ok((
        IdentityFixture {
            content,
            canonical_text,
            canonical_binary: binary,
        },
        length,
    ))
}

fn source_number(row: &TableRow, field: &'static str, name: &str) -> Result<u64, GoldenError> {
    let maximum = u64::try_from(MAX_SOURCE_BYTES).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: source bound cannot be represented: {source}"
        ))
    })?;
    decimal(row.field(field)?, &format!("{name} {field}"), maximum)
}

pub(super) fn digest(payload: &[u8]) -> Result<blake3::Hash, GoldenError> {
    let length = u64::try_from(payload.len()).map_err(|source| {
        GoldenError::violation(format!("payload length cannot be represented: {source}"))
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&DATA_MAGIC);
    hasher.update(&VERSION.to_be_bytes());
    hasher.update(&[ALGORITHM]);
    hasher.update(payload);
    hasher.update(&length.to_be_bytes());
    Ok(hasher.finalize())
}

pub(super) fn expected_text(length: u64, identity_digest: &blake3::Hash) -> String {
    format!(
        "keep:blob:v1:blake3-256:{length}:{}",
        identity_digest.to_hex()
    )
}

fn source_bytes(
    corpus: &Corpus,
    row: &TableRow,
    repetitions: u64,
    declared_length: u64,
) -> Result<Vec<u8>, GoldenError> {
    let name = row.field("case")?;
    let kind = row.field("source_kind")?;
    let parameter = row.field("source_parameter")?;
    let declared_usize = usize::try_from(declared_length).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: source length cannot fit this platform: {source}"
        ))
    })?;
    let content = match (kind, parameter, repetitions) {
        ("empty-v1", "-", 1) => Vec::new(),
        ("file-v1", _, 1) => {
            let source = corpus.source_file(parameter)?;
            if source.len() != declared_length {
                return Err(GoldenError::violation(format!(
                    "{name}: source size differs from its declaration"
                )));
            }
            source.bounded_bytes(MAX_SOURCE_BYTES, name)?
        }
        ("byte-ramp-v1", "-", count) if count > 0 => byte_ramp(name, count, declared_usize)?,
        _ => {
            return Err(GoldenError::violation(format!(
                "{name}: invalid source declaration"
            )));
        }
    };
    if content.len() == declared_usize {
        Ok(content)
    } else {
        Err(GoldenError::violation(format!(
            "{name}: source length mismatch"
        )))
    }
}

fn byte_ramp(name: &str, repetitions: u64, declared_length: usize) -> Result<Vec<u8>, GoldenError> {
    let expected = repetitions
        .checked_mul(256)
        .ok_or_else(|| GoldenError::violation(format!("{name}: byte-ramp length overflow")))?;
    let maximum = u64::try_from(MAX_SOURCE_BYTES).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: source bound cannot be represented: {source}"
        ))
    })?;
    let declared = u64::try_from(declared_length).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: declared length cannot be represented: {source}"
        ))
    })?;
    if expected > maximum || expected != declared {
        return Err(GoldenError::violation(format!(
            "{name}: byte-ramp length is invalid or unbounded"
        )));
    }
    let mut content = Vec::with_capacity(declared_length);
    content.extend((u8::MIN..=u8::MAX).cycle().take(declared_length));
    Ok(content)
}

fn expected_binary(length: u64, identity_digest: &blake3::Hash) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(BINARY_LENGTH);
    encoded.extend_from_slice(&ID_MAGIC);
    encoded.extend_from_slice(&VERSION.to_be_bytes());
    encoded.push(ALGORITHM);
    encoded.extend_from_slice(&length.to_be_bytes());
    encoded.extend_from_slice(identity_digest.as_bytes());
    encoded
}

fn check_required_identities(
    fixtures: &BTreeMap<String, IdentityFixture>,
) -> Result<(), GoldenError> {
    const REQUIRED: [&str; 6] = [
        "empty",
        "small-text",
        "binary-ramp",
        "large-ramp",
        "state-a",
        "state-b",
    ];
    if REQUIRED.iter().all(|name| fixtures.contains_key(*name)) {
        Ok(())
    } else {
        Err(GoldenError::violation(
            "identities.tsv: required v1 cases are absent",
        ))
    }
}

fn check_state_insertion(fixtures: &BTreeMap<String, IdentityFixture>) -> Result<(), GoldenError> {
    let state_a = fixtures
        .get("state-a")
        .ok_or_else(|| GoldenError::violation("identities.tsv: state-a is absent"))?;
    let state_b = fixtures
        .get("state-b")
        .ok_or_else(|| GoldenError::violation("identities.tsv: state-b is absent"))?;
    let (prefix, suffix) = state_a
        .content
        .split_at_checked(6)
        .ok_or_else(|| GoldenError::violation("state-a is shorter than insertion offset"))?;
    let expected_length = state_a
        .content
        .len()
        .checked_add(b"INSERTED\n".len())
        .ok_or_else(|| GoldenError::violation("state-b length overflow"))?;
    let mut expected = Vec::with_capacity(expected_length);
    expected.extend_from_slice(prefix);
    expected.extend_from_slice(b"INSERTED\n");
    expected.extend_from_slice(suffix);
    if state_b.content == expected {
        Ok(())
    } else {
        Err(GoldenError::violation(
            "state-b is not the declared insertion into state-a",
        ))
    }
}

#[cfg(test)]
mod tests;
