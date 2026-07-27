//! This module owns bounded dispatch across production corpus parsers.

use std::path::Path;

use super::GoldenError;
use super::canonical_value::{EmptyHex, case_name, decimal, decoded_hex};
use super::corpus_protocol::{MAX_SOURCE_BYTES, protocol_lines_from_bytes, table_rows};
use super::invalid_text_oracle::text_outcome;
use super::mutation_value::{mutate, mutation_offset};
use crate::protocol_admission::tab_fields;

const PARSER_COUNT: u8 = 9;

pub(super) fn admit(selector: u8, input: &[u8]) -> Result<(), GoldenError> {
    if input.len() > 1_048_576 {
        return Err(GoldenError::violation(
            "fuzz input exceeds the corpus protocol bound",
        ));
    }
    match selector.checked_rem(PARSER_COUNT) {
        Some(0) => admit_table(
            "identities.tsv",
            "# keep.golden-file-worldline.identities/v1",
            &[
                "case",
                "source_kind",
                "source_parameter",
                "repetitions",
                "logical_length",
                "canonical_text",
                "canonical_binary_hex",
            ],
            input,
        ),
        Some(1) => admit_table(
            "invalid-text.tsv",
            "# keep.golden-file-worldline.invalid-text/v1",
            &["case", "input_hex", "expected_outcome"],
            input,
        ),
        Some(2) => admit_table(
            "mutations.tsv",
            "# keep.golden-file-worldline.mutations/v1",
            &[
                "case",
                "target_kind",
                "target_case",
                "operation",
                "offset",
                "value_hex",
                "expected_outcome",
            ],
            input,
        ),
        Some(3) => admit_table(
            "steps.tsv",
            "# keep.golden-file-worldline.steps/v1",
            &[
                "step",
                "operation",
                "input_case",
                "identity_case",
                "expected_outcome",
            ],
            input,
        ),
        Some(4) => admit_table(
            "capabilities.tsv",
            "# keep.golden-file-worldline.capabilities/v1",
            &[
                "capability",
                "posture",
                "first_milestone",
                "owning_issues",
                "claim",
            ],
            input,
        ),
        Some(5) => admit_case(input),
        Some(6) => admit_decimal(input),
        Some(7) => text_outcome(input).map(drop),
        Some(8) => admit_mutation(input),
        Some(_) | None => Err(GoldenError::violation(
            "fuzz parser selector is unreachable",
        )),
    }
}

fn admit_table(
    table: &str,
    schema: &str,
    columns: &[&'static str],
    input: &[u8],
) -> Result<(), GoldenError> {
    let lines = protocol_lines_from_bytes(Path::new(table), input)?;
    table_rows(table, schema, columns, lines).map(drop)
}

fn admit_case(input: &[u8]) -> Result<(), GoldenError> {
    let value = bounded_utf8(input)?;
    case_name(value, "fuzz-input").map(drop)
}

fn admit_decimal(input: &[u8]) -> Result<(), GoldenError> {
    let value = bounded_utf8(input)?;
    decimal(value, "fuzz decimal", u64::MAX).map(drop)
}

fn admit_mutation(input: &[u8]) -> Result<(), GoldenError> {
    let fields: [&str; 4] = tab_fields(bounded_utf8(input)?, 4)
        .map_err(|_| GoldenError::violation("fuzz mutation field count is invalid"))?
        .try_into()
        .map_err(|_| GoldenError::violation("fuzz mutation field count is invalid"))?;
    let [target_hex, operation, offset, value_hex] = fields;
    let target = decoded_hex(
        target_hex,
        "fuzz mutation target",
        MAX_SOURCE_BYTES,
        EmptyHex::Refuse,
    )?;
    let parsed_offset = mutation_offset(offset, target.len(), "fuzz mutation")?;
    mutate(
        &target,
        operation,
        parsed_offset,
        value_hex,
        "fuzz mutation",
    )
    .map(drop)
}

fn bounded_utf8(input: &[u8]) -> Result<&str, GoldenError> {
    std::str::from_utf8(input)
        .map_err(|source| GoldenError::violation(format!("fuzz input is not UTF-8: {source}")))
}
