//! Scenario, text-failure, and mutation fixture loading.

use std::collections::BTreeSet;

use super::harness_failure::HarnessFailure;
use super::identity_corpus::{canonical_decimal, decode_hex};

const STEPS: &str = include_str!("../../conformance/golden-file-worldline/v1/steps.tsv");
const INVALID_TEXT: &str =
    include_str!("../../conformance/golden-file-worldline/v1/invalid-text.tsv");
const MUTATIONS: &str = include_str!("../../conformance/golden-file-worldline/v1/mutations.tsv");

pub(super) struct ScenarioStep {
    pub(super) operation: &'static str,
    pub(super) input_case: &'static str,
    pub(super) identity_case: &'static str,
    pub(super) expected_outcome: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TextErrorClass {
    InputTooLong,
    MissingField,
    TrailingData,
    InvalidScheme,
    InvalidKind,
    MalformedVersion,
    UnsupportedVersion,
    UnsupportedAlgorithm,
    NonCanonicalLength,
    LengthOverflow,
    InvalidDigestLength,
    NonCanonicalDigestCase,
    InvalidDigestAlphabet,
}

pub(super) struct InvalidTextCase {
    pub(super) input: Vec<u8>,
    pub(super) expected: TextErrorClass,
}

pub(super) struct MutationCase {
    pub(super) target_kind: &'static str,
    pub(super) target_case: &'static str,
    pub(super) operation: &'static str,
    pub(super) offset: usize,
    pub(super) value: Vec<u8>,
    pub(super) expected_outcome: &'static str,
}

pub(super) fn scenario_steps() -> Result<Vec<ScenarioStep>, HarnessFailure> {
    let mut lines = STEPS.lines();
    header(lines.next(), "# keep.golden-file-worldline.steps/v1")?;
    header(
        lines.next(),
        "step\toperation\tinput_case\tidentity_case\texpected_outcome",
    )?;
    lines.map(parse_step).collect()
}

pub(super) fn invalid_text_cases() -> Result<Vec<InvalidTextCase>, HarnessFailure> {
    let mut lines = INVALID_TEXT.lines();
    header(lines.next(), "# keep.golden-file-worldline.invalid-text/v1")?;
    header(lines.next(), "case\tinput_hex\texpected_outcome")?;
    lines.map(parse_invalid_text).collect()
}

pub(super) fn mutation_cases() -> Result<Vec<MutationCase>, HarnessFailure> {
    let mut lines = MUTATIONS.lines();
    header(lines.next(), "# keep.golden-file-worldline.mutations/v1")?;
    header(
        lines.next(),
        concat!(
            "case\ttarget_kind\ttarget_case\toperation\toffset\tvalue_hex\t",
            "expected_outcome"
        ),
    )?;
    let mut cases = Vec::new();
    let mut seen = BTreeSet::new();
    for line in lines {
        let (case, mutation) = parse_mutation(line)?;
        if !seen.insert(case) {
            return Err(HarnessFailure::corpus("duplicate mutation case"));
        }
        validate_mutation(&mutation)?;
        cases.push(mutation);
    }
    Ok(cases)
}

fn parse_step(line: &'static str) -> Result<ScenarioStep, HarnessFailure> {
    let mut fields = line.split('\t');
    let _step = field(&mut fields)?;
    let operation = field(&mut fields)?;
    let input_case = field(&mut fields)?;
    let identity_case = field(&mut fields)?;
    let expected_outcome = field(&mut fields)?;
    no_trailing(fields.next())?;
    Ok(ScenarioStep {
        operation,
        input_case,
        identity_case,
        expected_outcome,
    })
}

fn parse_invalid_text(line: &'static str) -> Result<InvalidTextCase, HarnessFailure> {
    let mut fields = line.split('\t');
    let _case = field(&mut fields)?;
    let input = decode_hex(field_allow_empty(&mut fields)?)?;
    let expected = text_error_class(field(&mut fields)?)?;
    no_trailing(fields.next())?;
    Ok(InvalidTextCase { input, expected })
}

fn parse_mutation(line: &'static str) -> Result<(&'static str, MutationCase), HarnessFailure> {
    let mut fields = line.split('\t');
    let case = field(&mut fields)?;
    let target_kind = field(&mut fields)?;
    let target_case = field(&mut fields)?;
    let operation = field(&mut fields)?;
    let offset = canonical_decimal(
        field(&mut fields)?,
        "mutation offset is noncanonical",
        "mutation offset is invalid",
    )?;
    let value_field = field(&mut fields)?;
    let value = if value_field == "-" {
        Vec::new()
    } else {
        decode_hex(value_field)?
    };
    let expected_outcome = field(&mut fields)?;
    no_trailing(fields.next())?;
    Ok((
        case,
        MutationCase {
            target_kind,
            target_case,
            operation,
            offset,
            value,
            expected_outcome,
        },
    ))
}

fn text_error_class(outcome: &str) -> Result<TextErrorClass, HarnessFailure> {
    match outcome {
        "keep.identity.input_too_long" => Ok(TextErrorClass::InputTooLong),
        "keep.identity.malformed_structure" => Ok(TextErrorClass::MissingField),
        "keep.identity.trailing_data" => Ok(TextErrorClass::TrailingData),
        "keep.identity.invalid_scheme" => Ok(TextErrorClass::InvalidScheme),
        "keep.identity.invalid_kind" => Ok(TextErrorClass::InvalidKind),
        "keep.identity.malformed_version" => Ok(TextErrorClass::MalformedVersion),
        "keep.identity.unsupported_version" => Ok(TextErrorClass::UnsupportedVersion),
        "keep.identity.unsupported_algorithm" => Ok(TextErrorClass::UnsupportedAlgorithm),
        "keep.identity.noncanonical_length" => Ok(TextErrorClass::NonCanonicalLength),
        "keep.identity.length_overflow" => Ok(TextErrorClass::LengthOverflow),
        "keep.identity.invalid_digest_length" => Ok(TextErrorClass::InvalidDigestLength),
        "keep.identity.noncanonical_digest_case" => Ok(TextErrorClass::NonCanonicalDigestCase),
        "keep.identity.invalid_digest_alphabet" => Ok(TextErrorClass::InvalidDigestAlphabet),
        _ => Err(HarnessFailure::corpus("unknown text error outcome")),
    }
}

fn validate_mutation(mutation: &MutationCase) -> Result<(), HarnessFailure> {
    let valid = matches!(
        (
            mutation.target_kind,
            mutation.operation,
            mutation.expected_outcome,
        ),
        (
            "content",
            "xor-byte" | "truncate" | "append",
            "keep.content.mismatch"
        ) | (
            "identity-binary",
            "xor-byte",
            "keep.identity.invalid_magic" | "keep.identity.different_supported_identity"
        ) | (
            "identity-binary",
            "set-u16-be",
            "keep.identity.unsupported_version"
        ) | (
            "identity-binary",
            "set-u8",
            "keep.identity.unsupported_algorithm"
        ) | ("identity-binary", "truncate", "keep.identity.truncated")
            | ("identity-binary", "append", "keep.identity.trailing_data")
    );
    if !valid {
        return Err(HarnessFailure::corpus("unsupported mutation declaration"));
    }
    let value_width_is_valid = match mutation.operation {
        "xor-byte" | "set-u8" => mutation.value.len() == 1,
        "set-u16-be" => mutation.value.len() == 2,
        "truncate" => mutation.value.is_empty(),
        "append" => !mutation.value.is_empty(),
        _ => false,
    };
    if value_width_is_valid {
        Ok(())
    } else {
        Err(HarnessFailure::corpus("invalid mutation value width"))
    }
}

fn field(fields: &mut std::str::Split<'static, char>) -> Result<&'static str, HarnessFailure> {
    let value = field_allow_empty(fields)?;
    if value.is_empty() {
        Err(HarnessFailure::corpus("scenario field is empty"))
    } else {
        Ok(value)
    }
}

fn field_allow_empty(
    fields: &mut std::str::Split<'static, char>,
) -> Result<&'static str, HarnessFailure> {
    fields
        .next()
        .ok_or_else(|| HarnessFailure::corpus("scenario row is missing a field"))
}

const fn no_trailing(field: Option<&str>) -> Result<(), HarnessFailure> {
    if field.is_some() {
        Err(HarnessFailure::corpus("scenario row has trailing fields"))
    } else {
        Ok(())
    }
}

fn header(observed: Option<&str>, expected: &str) -> Result<(), HarnessFailure> {
    if observed == Some(expected) {
        Ok(())
    } else {
        Err(HarnessFailure::corpus("scenario header mismatch"))
    }
}
