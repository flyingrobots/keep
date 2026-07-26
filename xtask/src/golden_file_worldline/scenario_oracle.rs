use std::collections::{BTreeMap, BTreeSet};

use super::canonical_value::decimal;
use super::identity_oracle::IdentityFixture;
use super::{Corpus, GoldenError};

const STEP_COLUMNS: [&str; 5] = [
    "step",
    "operation",
    "input_case",
    "identity_case",
    "expected_outcome",
];
const REQUIRED_STEP_OPERATIONS: [&str; 10] = [
    "identify",
    "admit-exact",
    "read-exact",
    "identify",
    "admit-exact",
    "read-exact",
    "read-exact",
    "verify-claimed-content",
    "read-exact",
    "read-absent",
];
pub(super) fn check_steps(
    corpus: &Corpus,
    fixtures: &BTreeMap<String, IdentityFixture>,
) -> Result<(), GoldenError> {
    let rows = corpus.rows(
        "steps.tsv",
        "# keep.golden-file-worldline.steps/v1",
        &STEP_COLUMNS,
    )?;
    let maximum = u64::try_from(rows.len()).map_err(|source| {
        GoldenError::violation(format!(
            "scenario row count cannot be represented: {source}"
        ))
    })?;
    let mut admitted = BTreeSet::new();
    let mut operations = Vec::new();
    for (offset, row) in rows.into_iter().enumerate() {
        let expected_number = offset
            .checked_add(1)
            .ok_or_else(|| GoldenError::violation("scenario step number overflow"))?;
        let number = decimal(row.field("step")?, "scenario step", maximum)?;
        let expected_u64 = u64::try_from(expected_number).map_err(|source| {
            GoldenError::violation(format!("scenario step cannot be represented: {source}"))
        })?;
        if number != expected_u64 {
            return Err(GoldenError::violation(
                "steps.tsv: step numbers are not canonical and contiguous",
            ));
        }
        let operation = row.field("operation")?;
        let expected_outcome = step_outcome(operation).ok_or_else(|| {
            GoldenError::violation(format!("steps.tsv:{number}: operation outcome is invalid"))
        })?;
        if row.field("expected_outcome")? != expected_outcome {
            return Err(GoldenError::violation(format!(
                "steps.tsv:{number}: operation outcome is invalid"
            )));
        }
        if operation == "read-absent" {
            let identity = fixtures.get(row.field("identity_case")?).ok_or_else(|| {
                GoldenError::violation(format!("steps.tsv:{number}: identity case is absent"))
            })?;
            if row.field("input_case")? != "-" || admitted.contains(&identity.canonical_binary) {
                return Err(GoldenError::violation(format!(
                    "steps.tsv:{number}: absent read is not absent"
                )));
            }
        } else {
            check_exact_step(&row, fixtures, operation, number, &mut admitted)?;
        }
        operations.push(operation.to_owned());
    }
    if operations
        .iter()
        .map(String::as_str)
        .eq(REQUIRED_STEP_OPERATIONS)
    {
        Ok(())
    } else {
        Err(GoldenError::violation(
            "steps.tsv: ordered Golden File Worldline v1 operations moved",
        ))
    }
}

fn check_exact_step(
    row: &super::corpus_protocol::TableRow,
    fixtures: &BTreeMap<String, IdentityFixture>,
    operation: &str,
    number: u64,
    admitted: &mut BTreeSet<Vec<u8>>,
) -> Result<(), GoldenError> {
    let identity = fixtures.get(row.field("identity_case")?).ok_or_else(|| {
        GoldenError::violation(format!("steps.tsv:{number}: identity case is absent"))
    })?;
    let source = fixtures.get(row.field("input_case")?).ok_or_else(|| {
        GoldenError::violation(format!("steps.tsv:{number}: input case is absent"))
    })?;
    let same_identity = source.canonical_binary == identity.canonical_binary;
    if operation == "verify-claimed-content" && same_identity {
        return Err(GoldenError::violation(format!(
            "steps.tsv:{number}: mismatch uses matching content"
        )));
    }
    if operation != "verify-claimed-content" && !same_identity {
        return Err(GoldenError::violation(format!(
            "steps.tsv:{number}: exact operation substitutes identity"
        )));
    }
    if operation == "admit-exact" {
        admitted.insert(identity.canonical_binary.clone());
    }
    if operation == "read-exact" && !admitted.contains(&identity.canonical_binary) {
        return Err(GoldenError::violation(format!(
            "steps.tsv:{number}: exact read precedes admission"
        )));
    }
    Ok(())
}

fn step_outcome(operation: &str) -> Option<&'static str> {
    match operation {
        "identify" => Some("keep.identity.identified"),
        "admit-exact" => Some("keep.content.admitted"),
        "read-exact" => Some("keep.content.exact"),
        "verify-claimed-content" => Some("keep.content.mismatch"),
        "read-absent" => Some("keep.content.absent"),
        _ => None,
    }
}
