//! This module owns bounded corruption mutations and their expected outcomes.

use std::collections::{BTreeMap, BTreeSet};

use super::canonical_value::{case_name, unique};
use super::corpus_protocol::TableRow;
use super::digest_port::IdentityDigestOracle;
use super::identity_oracle::{
    ALGORITHM, BINARY_LENGTH, ID_MAGIC, IdentityFixture, VERSION, expected_text, verified_digest,
};
use super::mutation_value::{mutate, mutation_offset};
use super::{Corpus, GoldenError};

#[cfg(test)]
use super::mutation_value::apply_fixed_width;

const MUTATION_COLUMNS: [&str; 7] = [
    "case",
    "target_kind",
    "target_case",
    "operation",
    "offset",
    "value_hex",
    "expected_outcome",
];
const REQUIRED_OPERATIONS: [(&str, &str); 8] = [
    ("content", "xor-byte"),
    ("content", "truncate"),
    ("content", "append"),
    ("identity-binary", "xor-byte"),
    ("identity-binary", "truncate"),
    ("identity-binary", "append"),
    ("identity-binary", "set-u8"),
    ("identity-binary", "set-u16-be"),
];

pub(super) fn check(
    corpus: &Corpus,
    fixtures: &BTreeMap<String, IdentityFixture>,
    oracle: &impl IdentityDigestOracle,
) -> Result<(), GoldenError> {
    let rows = corpus.rows(
        "mutations.tsv",
        "# keep.golden-file-worldline.mutations/v1",
        &MUTATION_COLUMNS,
    )?;
    let mut seen = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for row in rows {
        let name = case_name(row.field("case")?, "mutations.tsv")?.to_owned();
        unique(&name, &mut seen, "mutations.tsv")?;
        let (target_kind, operation) = check_mutation(fixtures, &row, &name, oracle)?;
        covered.insert((target_kind, operation));
    }
    if REQUIRED_OPERATIONS
        .iter()
        .all(|(kind, operation)| covered.contains(&(String::from(*kind), String::from(*operation))))
    {
        Ok(())
    } else {
        Err(GoldenError::violation(
            "mutations.tsv: required v1 mutation coverage is absent",
        ))
    }
}

fn check_mutation(
    fixtures: &BTreeMap<String, IdentityFixture>,
    row: &TableRow,
    name: &str,
    oracle: &impl IdentityDigestOracle,
) -> Result<(String, String), GoldenError> {
    let target_kind = row.field("target_kind")?;
    if !matches!(target_kind, "content" | "identity-binary") {
        return Err(GoldenError::violation(format!(
            "{name}: unknown mutation target kind"
        )));
    }
    let fixture = fixtures
        .get(row.field("target_case")?)
        .ok_or_else(|| GoldenError::violation(format!("{name}: mutation target case is absent")))?;
    let target = target_bytes(fixture, target_kind);
    let offset = mutation_offset(row.field("offset")?, target.len(), name)?;
    let operation = row.field("operation")?;
    let changed = mutate(target, operation, offset, row.field("value_hex")?, name)?;
    check_outcome(fixture, target_kind, &changed, row, oracle)?;
    Ok((target_kind.to_owned(), operation.to_owned()))
}

fn target_bytes<'a>(fixture: &'a IdentityFixture, target_kind: &str) -> &'a [u8] {
    if target_kind == "content" {
        &fixture.content
    } else {
        &fixture.canonical_binary
    }
}

fn check_outcome(
    fixture: &IdentityFixture,
    target_kind: &str,
    changed: &[u8],
    row: &TableRow,
    oracle: &impl IdentityDigestOracle,
) -> Result<(), GoldenError> {
    let expected_outcome = row.field("expected_outcome")?;
    let name = row.field("case")?;
    if target_kind == "content" {
        let length = u64::try_from(changed.len()).map_err(|source| {
            GoldenError::violation(format!(
                "{name}: mutation length cannot be represented: {source}"
            ))
        })?;
        let observed = expected_text(length, &verified_digest(changed, oracle)?);
        if expected_outcome == "keep.content.mismatch" && observed != fixture.canonical_text {
            return Ok(());
        }
    } else if expected_outcome == binary_outcome(changed)? {
        return Ok(());
    }
    Err(GoldenError::violation(format!(
        "{name}: mutation outcome is incorrect"
    )))
}

fn binary_outcome(encoded: &[u8]) -> Result<&'static str, GoldenError> {
    if encoded.len() < BINARY_LENGTH {
        return Ok("keep.identity.truncated");
    }
    if encoded.len() > BINARY_LENGTH {
        return Ok("keep.identity.trailing_data");
    }
    if encoded.get(..ID_MAGIC.len()) != Some(ID_MAGIC.as_slice()) {
        return Ok("keep.identity.invalid_magic");
    }
    let version_start = ID_MAGIC.len();
    let version_end = version_start
        .checked_add(2)
        .ok_or_else(|| GoldenError::violation("identity version offset overflow"))?;
    let version_bytes: [u8; 2] = encoded
        .get(version_start..version_end)
        .ok_or_else(|| GoldenError::violation("identity version is absent"))?
        .try_into()
        .map_err(|source| {
            GoldenError::violation(format!("identity version width is invalid: {source}"))
        })?;
    if u16::from_be_bytes(version_bytes) != VERSION {
        return Ok("keep.identity.unsupported_version");
    }
    if encoded.get(version_end) != Some(&ALGORITHM) {
        return Ok("keep.identity.unsupported_algorithm");
    }
    Ok("keep.identity.different_supported_identity")
}

#[cfg(test)]
mod tests;
