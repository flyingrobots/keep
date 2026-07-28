//! This module owns independent admission and verification of `ChunkId` v1 vectors.

use std::collections::BTreeSet;
use std::path::Path;

use xtask::protocol_admission::EmptyHex;

use super::ConformanceError;
use super::canonical::{case_name, decimal, exact_hex, lower_hex};
use super::corpus::{Corpus, TablePolicy, TableRow};
use super::external_digest;

const ALGORITHM: [u8; 1] = [1];
const COLUMNS: [&str; 6] = [
    "case",
    "recipe",
    "parameter",
    "count",
    "chunk_length",
    "digest_hex",
];
const DATA_MAGIC: [u8; 16] = *b"KEEP:CHUNK:DATA\0";
const EXPECTED_TOTAL_BYTES: usize = 262_163;
const MAX_CASES: usize = 16;
const MAX_CHUNK_BYTES: usize = 262_144;
const MAX_TABLE_BYTES: usize = 1_048_576;
const MAX_TOTAL_BYTES: usize = 300_000;
const REQUIRED_CASES: [&str; 3] = ["maximum-zeros", "one-zero", "sample-text"];
const SCHEMA: &str = "keep.chunk-identities/v1";
const TABLE_POLICY: TablePolicy = TablePolicy::new(SCHEMA, &COLUMNS, MAX_TABLE_BYTES, MAX_CASES);
const VERSION: [u8; 2] = 1_u16.to_be_bytes();

struct IdentityCase {
    name: String,
    payload: Vec<u8>,
    expected_digest: [u8; 32],
}

pub(super) fn check(repository_root: &Path) -> Result<(), ConformanceError> {
    let corpus = Corpus::open(repository_root.join("conformance/chunk-id/v1"))?;
    for case in read_cases(&corpus)? {
        let observed = digest(&case.payload)?;
        if observed != case.expected_digest {
            return Err(ConformanceError::violation(format!(
                "ChunkId digest moved for {}",
                case.name
            )));
        }
    }
    Ok(())
}

fn read_cases(corpus: &Corpus) -> Result<Vec<IdentityCase>, ConformanceError> {
    let rows = corpus.rows("identities.tsv", TABLE_POLICY)?;
    let mut cases = Vec::with_capacity(rows.len());
    let mut names = BTreeSet::new();
    let mut total = 0_usize;
    for row in rows {
        let case = read_case(&row)?;
        if !names.insert(case.name.clone()) {
            return Err(ConformanceError::violation(
                "identity case name is duplicated",
            ));
        }
        total = total
            .checked_add(case.payload.len())
            .ok_or_else(|| ConformanceError::violation("identity corpus byte count overflow"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(ConformanceError::violation(
                "identity corpus exceeds its aggregate bound",
            ));
        }
        cases.push(case);
    }
    let required = REQUIRED_CASES.into_iter().collect::<BTreeSet<_>>();
    let observed = names.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if observed != required || total != EXPECTED_TOTAL_BYTES {
        return Err(ConformanceError::violation(
            "required identity witnesses or aggregate length moved",
        ));
    }
    Ok(cases)
}

fn read_case(row: &TableRow) -> Result<IdentityCase, ConformanceError> {
    let name = row.field("case")?;
    case_name(name, "identities.tsv")?;
    let count = decimal(row.field("count")?, "recipe count", MAX_CHUNK_BYTES)?;
    let declared = decimal(row.field("chunk_length")?, "chunk length", MAX_CHUNK_BYTES)?;
    let payload = payload_for(row.field("recipe")?, row.field("parameter")?, count)?;
    if payload.is_empty() || payload.len() != declared {
        return Err(ConformanceError::violation(
            "payload length does not match the nonempty declaration",
        ));
    }
    let expected_digest = exact_hex(row.field("digest_hex")?, "expected digest", 32)?
        .try_into()
        .map_err(|_| ConformanceError::violation("expected digest has the wrong width"))?;
    Ok(IdentityCase {
        name: name.to_owned(),
        payload,
        expected_digest,
    })
}

fn payload_for(recipe: &str, parameter: &str, count: usize) -> Result<Vec<u8>, ConformanceError> {
    let pattern = match recipe {
        "repeated-byte-v1" => exact_hex(parameter, "repeated-byte parameter", 1)?,
        "hex-repeat-v1" => lower_hex(
            parameter,
            "hex-repeat parameter",
            MAX_CHUNK_BYTES,
            EmptyHex::Refuse,
        )?,
        _ => {
            return Err(ConformanceError::violation(format!(
                "unsupported recipe {recipe:?}"
            )));
        }
    };
    let declared = pattern
        .len()
        .checked_mul(count)
        .ok_or_else(|| ConformanceError::violation("recipe length overflow"))?;
    if declared > MAX_CHUNK_BYTES {
        return Err(ConformanceError::violation(
            "recipe exceeds the fixture chunk bound",
        ));
    }
    Ok(pattern.repeat(count))
}

fn digest(payload: &[u8]) -> Result<[u8; 32], ConformanceError> {
    let length = u32::try_from(payload.len())
        .map_err(|source| ConformanceError::violation(format!("chunk length overflow: {source}")))?
        .to_be_bytes();
    external_digest::digest(&[
        DATA_MAGIC.as_slice(),
        VERSION.as_slice(),
        ALGORITHM.as_slice(),
        payload,
        length.as_slice(),
    ])
}

#[cfg(test)]
mod tests {
    use super::{ConformanceError, case_name, decimal, payload_for};

    #[test]
    fn ambiguous_decimal_and_identifier_spellings_are_refused() {
        assert!(matches!(
            decimal("01", "recipe count", 16),
            Err(ConformanceError::Violation(ref message))
                if message == "recipe count: noncanonical unsigned decimal \"01\""
        ));
        assert!(matches!(
            case_name("two--hyphens", "identities.tsv"),
            Err(ConformanceError::Violation(ref message))
                if message == "identities.tsv: noncanonical case name \"two--hyphens\""
        ));
    }

    #[test]
    fn recipe_expansion_refuses_before_crossing_the_chunk_bound() {
        assert!(matches!(
            payload_for("hex-repeat-v1", "0001", 131_073),
            Err(ConformanceError::Violation(ref message))
                if message == "recipe exceeds the fixture chunk bound"
        ));
    }
}
