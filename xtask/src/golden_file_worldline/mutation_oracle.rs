use std::collections::{BTreeMap, BTreeSet};

use super::canonical_value::{EmptyHex, case_name, decimal, decoded_hex, unique};
use super::corpus_protocol::{MAX_MUTATION_VALUE_BYTES, MAX_SOURCE_BYTES, TableRow};
use super::identity_oracle::{
    ALGORITHM, BINARY_LENGTH, ID_MAGIC, IdentityFixture, VERSION, digest, expected_text,
};
use super::{Corpus, GoldenError};

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
) -> Result<(), GoldenError> {
    let rows = corpus.rows(
        "mutations.tsv",
        "# keep.golden-file-worldline.mutations/v1",
        &MUTATION_COLUMNS,
    )?;
    let mut seen = BTreeSet::new();
    let mut covered = BTreeSet::new();
    for row in rows {
        let (name, target_kind, operation) = check_mutation(fixtures, &row)?;
        unique(&name, &mut seen, "mutations.tsv")?;
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
) -> Result<(String, String, String), GoldenError> {
    let name = case_name(row.field("case")?, "mutations.tsv")?;
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
    check_outcome(
        fixture,
        target_kind,
        &changed,
        row.field("expected_outcome")?,
        name,
    )?;
    Ok((
        name.to_owned(),
        target_kind.to_owned(),
        operation.to_owned(),
    ))
}

fn mutation_offset(value: &str, target_length: usize, name: &str) -> Result<usize, GoldenError> {
    let maximum = u64::try_from(target_length).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: target length cannot be represented: {source}"
        ))
    })?;
    let offset = decimal(value, &format!("{name} offset"), maximum)?;
    usize::try_from(offset).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: mutation offset cannot fit this platform: {source}"
        ))
    })
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
    expected_outcome: &str,
    name: &str,
) -> Result<(), GoldenError> {
    if target_kind == "content" {
        let length = u64::try_from(changed.len()).map_err(|source| {
            GoldenError::violation(format!(
                "{name}: mutation length cannot be represented: {source}"
            ))
        })?;
        let observed = expected_text(length, &digest(changed)?);
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

fn mutate(
    target: &[u8],
    operation: &str,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<Vec<u8>, GoldenError> {
    let mut changed = target.to_vec();
    match operation {
        "truncate" => truncate(&mut changed, offset, value_field, name)?,
        "append" => append(&mut changed, offset, value_field, name)?,
        "xor-byte" | "set-u8" | "set-u16-be" => {
            apply_fixed_width(&mut changed, operation, offset, value_field, name)?;
        }
        _ => {
            return Err(GoldenError::violation(format!(
                "{name}: unknown mutation operation {operation:?}"
            )));
        }
    }
    let maximum = MAX_SOURCE_BYTES
        .checked_add(MAX_MUTATION_VALUE_BYTES)
        .ok_or_else(|| GoldenError::violation("mutation bound overflow"))?;
    if changed == target || changed.len() > maximum {
        Err(GoldenError::violation(format!(
            "{name}: mutation is a no-op or exceeds its bound"
        )))
    } else {
        Ok(changed)
    }
}

fn truncate(
    changed: &mut Vec<u8>,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<(), GoldenError> {
    if value_field != "-" || offset >= changed.len() {
        return Err(GoldenError::violation(format!(
            "{name}: invalid truncation"
        )));
    }
    changed.truncate(offset);
    Ok(())
}

fn append(
    changed: &mut Vec<u8>,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<(), GoldenError> {
    let value = decoded_hex(
        value_field,
        &format!("{name} value"),
        MAX_MUTATION_VALUE_BYTES,
        EmptyHex::Refuse,
    )?;
    if offset != changed.len() {
        return Err(GoldenError::violation(format!(
            "{name}: append offset does not equal target length"
        )));
    }
    changed.extend_from_slice(&value);
    Ok(())
}

fn apply_fixed_width(
    changed: &mut [u8],
    operation: &str,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<(), GoldenError> {
    let width = if operation == "set-u16-be" { 2 } else { 1 };
    let value = decoded_hex(
        value_field,
        &format!("{name} value"),
        width,
        EmptyHex::Refuse,
    )?;
    let end = offset
        .checked_add(width)
        .ok_or_else(|| GoldenError::violation(format!("{name}: mutation offset overflow")))?;
    let destination = changed.get_mut(offset..end).ok_or_else(|| {
        GoldenError::violation(format!("{name}: mutation width or offset is invalid"))
    })?;
    if operation == "xor-byte" {
        xor_byte(destination, &value, name)?;
    } else {
        destination.copy_from_slice(&value);
    }
    Ok(())
}

fn xor_byte(destination: &mut [u8], value: &[u8], name: &str) -> Result<(), GoldenError> {
    let [destination] = destination else {
        return Err(GoldenError::violation(format!(
            "{name}: mutation width is invalid"
        )));
    };
    let [value] = value else {
        return Err(GoldenError::violation(format!(
            "{name}: mutation value is invalid"
        )));
    };
    *destination ^= *value;
    Ok(())
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
