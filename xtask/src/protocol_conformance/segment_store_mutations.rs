//! This module owns the shape law of the segment-store mutation ledgers:
//! every row names a frozen fixture, a registered operation, posture,
//! record, stage, and requirement, and a span inside that fixture.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::ConformanceError;
use super::corpus::{Corpus, TablePolicy};

const COLUMNS: [&str; 11] = [
    "case",
    "record",
    "base_fixture",
    "operation",
    "offset",
    "span_length",
    "parameter",
    "checksum_posture",
    "expected_outcome",
    "stage",
    "requirement",
];
const OPERATIONS: [&str; 5] = [
    "replace-v1",
    "xor-v1",
    "truncate-v1",
    "append-v1",
    "delete-v1",
];
const POSTURES: [&str; 4] = [
    "preserve-v1",
    "recompute-v1",
    "recompute-trailer-v1",
    "recompute-checksum-v1",
];
const STAGES: [&str; 4] = ["framing", "checksum", "identity", "binding"];
const V1_RECORDS: [&str; 9] = [
    "segment-header",
    "segment-record",
    "segment-seal",
    "segment",
    "catalog",
    "catalog-entry",
    "catalog-binding",
    "publication-head",
    "head-binding",
];
const V2_RECORDS: [&str; 9] = [
    "format-marker",
    "migration-intent",
    "migration-receipt",
    "retention-root",
    "retention-manifest",
    "retention-head",
    "gc-intent",
    "gc-receipt",
    "disposition",
];
const MAXIMUM_TABLE_BYTES: usize = 1 << 20;
const MAXIMUM_ROWS: usize = 4_096;
const MAXIMUM_FIXTURE_BYTES: usize = 1 << 24;

/// Verifies both segment-store mutation ledgers against their fixtures.
///
/// The check performs bounded, blocking repository reads and no external
/// process. The executable law that every row reaches its named refusal is
/// `tests/segment_store_mutations.rs`; this check refuses a ledger that law
/// could not even apply.
pub(super) fn check(repository_root: &Path) -> Result<(), ConformanceError> {
    check_ledger(
        repository_root,
        "conformance/segment-store/v1",
        "keep.segment-store-mutations/v1",
        &V1_RECORDS,
    )?;
    check_ledger(
        repository_root,
        "conformance/segment-store/v2",
        "keep.segment-store-mutations/v2",
        &V2_RECORDS,
    )
}

fn check_ledger(
    repository_root: &Path,
    directory: &str,
    schema: &'static str,
    records: &[&str],
) -> Result<(), ConformanceError> {
    let corpus = Corpus::open(repository_root.join(directory))?;
    let rows = corpus.rows(
        "mutations.tsv",
        TablePolicy::new(schema, &COLUMNS, MAXIMUM_TABLE_BYTES, MAXIMUM_ROWS),
    )?;
    let mut cases = BTreeSet::new();
    let mut covered = BTreeSet::new();
    let mut fixture_lengths: BTreeMap<String, usize> = BTreeMap::new();
    for row in &rows {
        let case = row.field("case")?;
        if !cases.insert(case.to_owned()) {
            return Err(violation(schema, case, "duplicate case"));
        }
        let record = row.field("record")?;
        if !records.contains(&record) {
            return Err(violation(schema, case, "unregistered record"));
        }
        covered.insert(record.to_owned());
        require_member(
            schema,
            case,
            row.field("operation")?,
            &OPERATIONS,
            "operation",
        )?;
        require_member(
            schema,
            case,
            row.field("checksum_posture")?,
            &POSTURES,
            "posture",
        )?;
        require_member(schema, case, row.field("stage")?, &STAGES, "stage")?;
        require_outcome(schema, case, row.field("expected_outcome")?, records)?;
        require_requirement(schema, case, row.field("requirement")?)?;
        let fixture = row.field("base_fixture")?;
        let length = if let Some(length) = fixture_lengths.get(fixture) {
            *length
        } else {
            let length = fixture_length(&corpus, fixture)?;
            fixture_lengths.insert(fixture.to_owned(), length);
            length
        };
        require_span(schema, case, row, length)?;
    }
    if let Some(missing) = records.iter().find(|record| !covered.contains(**record)) {
        return Err(ConformanceError::violation(format!(
            "{schema}: record {missing} has no mutation"
        )));
    }
    Ok(())
}

fn violation(schema: &str, case: &str, message: &str) -> ConformanceError {
    ConformanceError::violation(format!("{schema}: {case}: {message}"))
}

fn require_member(
    schema: &str,
    case: &str,
    value: &str,
    registered: &[&str],
    what: &str,
) -> Result<(), ConformanceError> {
    if registered.contains(&value) {
        Ok(())
    } else {
        Err(violation(
            schema,
            case,
            &format!("unregistered {what} {value:?}"),
        ))
    }
}

fn require_outcome(
    schema: &str,
    case: &str,
    outcome: &str,
    records: &[&str],
) -> Result<(), ConformanceError> {
    let Some((record, variant)) = outcome.split_once('.') else {
        return Err(violation(schema, case, "outcome lacks a record prefix"));
    };
    let canonical = |text: &str| {
        !text.is_empty()
            && text
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    };
    if records.contains(&record) && canonical(variant) {
        Ok(())
    } else {
        Err(violation(schema, case, "outcome is not <record>.<variant>"))
    }
}

fn require_requirement(
    schema: &str,
    case: &str,
    requirement: &str,
) -> Result<(), ConformanceError> {
    let Some(rest) = requirement.strip_prefix("KEEP-") else {
        return Err(violation(
            schema,
            case,
            "requirement is not a KEEP identifier",
        ));
    };
    let Some((family, ordinal)) = rest.rsplit_once('-') else {
        return Err(violation(
            schema,
            case,
            "requirement is not a KEEP identifier",
        ));
    };
    if !family.is_empty()
        && family.bytes().all(|byte| byte.is_ascii_uppercase())
        && ordinal.len() == 3
        && ordinal.bytes().all(|byte| byte.is_ascii_digit())
    {
        Ok(())
    } else {
        Err(violation(
            schema,
            case,
            "requirement is not a KEEP identifier",
        ))
    }
}

fn fixture_length(corpus: &Corpus, fixture: &str) -> Result<usize, ConformanceError> {
    if fixture.strip_suffix(".hex").is_none() {
        return Err(ConformanceError::violation(format!(
            "mutation ledger names a non-hexadecimal fixture {fixture:?}"
        )));
    }
    let bytes = corpus
        .source_file(fixture)?
        .bounded_bytes(MAXIMUM_FIXTURE_BYTES, fixture)?;
    let text = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
    if text.len() % 2 != 0
        || !text
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        return Err(ConformanceError::violation(format!(
            "fixture {fixture} is not lowercase hexadecimal"
        )));
    }
    Ok(text.len().div_euclid(2))
}

fn require_span(
    schema: &str,
    case: &str,
    row: &super::corpus::TableRow,
    length: usize,
) -> Result<(), ConformanceError> {
    let offset = decimal(schema, case, row.field("offset")?)?;
    let span = decimal(schema, case, row.field("span_length")?)?;
    let parameter = row.field("parameter")?;
    let end = offset
        .checked_add(span)
        .ok_or_else(|| violation(schema, case, "span overflows"))?;
    let parameter_width = if parameter == "-" {
        None
    } else if parameter.len() % 2 == 0
        && parameter
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Some(parameter.len().div_euclid(2))
    } else {
        return Err(violation(
            schema,
            case,
            "parameter is not lowercase hexadecimal",
        ));
    };
    let lawful = match row.field("operation")? {
        "replace-v1" | "xor-v1" => end <= length && parameter_width == Some(span) && span > 0,
        "truncate-v1" => span == 0 && offset < length && parameter_width.is_none(),
        "append-v1" => span == 0 && offset == length && parameter_width.is_some_and(|w| w > 0),
        "delete-v1" => span > 0 && end <= length && parameter_width.is_none(),
        _ => false,
    };
    if lawful {
        Ok(())
    } else {
        Err(violation(
            schema,
            case,
            "span or parameter does not fit its operation",
        ))
    }
}

fn decimal(schema: &str, case: &str, text: &str) -> Result<usize, ConformanceError> {
    if text != "0" && text.starts_with('0') {
        return Err(violation(schema, case, "integer is not canonical decimal"));
    }
    text.parse()
        .map_err(|_source| violation(schema, case, "integer is not canonical decimal"))
}
