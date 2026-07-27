//! Public canonical `LayoutId` codec and mismatch laws.

pub mod support;

use std::error::Error;
use std::io;
use std::str;

use keep::{LayoutId, LayoutIdBinaryParseError, LayoutIdMismatch, LayoutIdTextParseError};
use support::{decode_hex, field, field_unchecked, invalid_corpus};

const LAYOUTS: &str = include_str!("../conformance/layout/v1/layouts.tsv");
const INVALID_BINARY: &str = include_str!("../conformance/layout/v1/invalid-layout-id-binary.tsv");
const INVALID_TEXT: &str = include_str!("../conformance/layout/v1/invalid-layout-id-text.tsv");

#[test]
fn every_golden_layout_identity_round_trips_both_coordinates() -> Result<(), Box<dyn Error>> {
    for line in LAYOUTS.lines().skip(2) {
        let case = field(line, 0)?;
        let text = field(line, 10)?;
        let binary = decode_hex(field(line, 11)?)?;
        let from_text = text.parse::<LayoutId>()?;
        let from_binary = LayoutId::parse_binary(&binary)?;

        assert_eq!(from_text, from_binary, "{case}");
        assert_eq!(from_text.to_string(), text, "{case}");
        assert_eq!(from_text.encode_binary().as_slice(), binary, "{case}");
        assert_eq!(
            from_text.plan_length().get().to_string(),
            field(line, 8)?,
            "{case}"
        );
    }
    Ok(())
}

#[test]
fn every_invalid_text_coordinate_has_its_exact_typed_refusal() -> Result<(), Box<dyn Error>> {
    for line in INVALID_TEXT.lines().skip(2) {
        let case = field(line, 0)?;
        let bytes = decode_hex(field(line, 1)?)?;
        let encoded = str::from_utf8(&bytes)?;
        let expected = field(line, 2)?;
        let error = encoded
            .parse::<LayoutId>()
            .err()
            .ok_or_else(|| invalid_corpus("invalid text coordinate was admitted"))?;

        assert_eq!(text_error_code(error), expected, "{case}");
    }
    Ok(())
}

#[test]
fn every_invalid_binary_coordinate_has_its_exact_typed_refusal() -> Result<(), Box<dyn Error>> {
    for line in INVALID_BINARY.lines().skip(2) {
        let case = field(line, 0)?;
        let base = golden_binary(field(line, 1)?)?;
        let expected_id = LayoutId::parse_binary(&base)?;
        let mutated = apply_mutation(&base, line)?;
        let expected = field(line, 6)?;
        let observed = match LayoutId::parse_binary(&mutated) {
            Ok(parsed) => parsed
                .verify_expected(expected_id)
                .err()
                .map(mismatch_code)
                .ok_or_else(|| invalid_corpus("binary mutation preserved the identity"))?,
            Err(error) => binary_error_code(error),
        };

        assert_eq!(observed, expected, "{case}");
    }
    Ok(())
}

#[test]
fn binary_insert_mutation_refuses_a_nonzero_span() -> Result<(), io::Error> {
    let malformed = "stray-span\tempty\tinsert-v1\t0\t1\t00\tunused";
    let error = apply_mutation(&[0_u8], malformed)
        .err()
        .ok_or_else(|| invalid_corpus("insert mutation admitted a nonzero span"))?;

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(error.to_string(), "insert mutation has a nonzero span");
    Ok(())
}

fn golden_binary(case_name: &str) -> Result<Vec<u8>, io::Error> {
    let line = LAYOUTS
        .lines()
        .skip(2)
        .find(|line| field_unchecked(line, 0) == Some(case_name))
        .ok_or_else(|| invalid_corpus("binary mutation names an unknown base case"))?;
    decode_hex(field(line, 11)?)
}

fn apply_mutation(base: &[u8], row: &str) -> Result<Vec<u8>, io::Error> {
    let operation = field(row, 2)?;
    let offset = field(row, 3)?
        .parse::<usize>()
        .map_err(|_source| invalid_corpus("mutation offset is not usize"))?;
    let span_length = field(row, 4)?
        .parse::<usize>()
        .map_err(|_source| invalid_corpus("mutation span is not usize"))?;
    let parameter = field(row, 5)?;
    let mut result = base.to_vec();
    let end = offset
        .checked_add(span_length)
        .ok_or_else(|| invalid_corpus("mutation range overflow"))?;

    match operation {
        "replace-v1" => replace(&mut result, offset, end, parameter)?,
        "xor-v1" => xor(&mut result, offset, end, parameter)?,
        "delete-v1" => delete(&mut result, offset, end, parameter)?,
        "insert-v1" => insert(&mut result, offset, span_length, parameter)?,
        _ => return Err(invalid_corpus("unknown identity mutation operation")),
    }
    Ok(result)
}

fn replace(bytes: &mut [u8], offset: usize, end: usize, parameter: &str) -> Result<(), io::Error> {
    let replacement = decode_hex(parameter)?;
    let target = bytes
        .get_mut(offset..end)
        .ok_or_else(|| invalid_corpus("replace range is out of bounds"))?;
    if target.len() != replacement.len() {
        return Err(invalid_corpus("replace width differs from its span"));
    }
    target.copy_from_slice(&replacement);
    Ok(())
}

fn xor(bytes: &mut [u8], offset: usize, end: usize, parameter: &str) -> Result<(), io::Error> {
    let mask = decode_hex(parameter)?;
    let target = bytes
        .get_mut(offset..end)
        .ok_or_else(|| invalid_corpus("xor range is out of bounds"))?;
    if target.len() != mask.len() {
        return Err(invalid_corpus("xor width differs from its span"));
    }
    for (byte, mask_byte) in target.iter_mut().zip(mask) {
        *byte ^= mask_byte;
    }
    Ok(())
}

fn delete(
    bytes: &mut Vec<u8>,
    offset: usize,
    end: usize,
    parameter: &str,
) -> Result<(), io::Error> {
    if parameter != "-" || bytes.get(offset..end).is_none() {
        return Err(invalid_corpus("invalid delete mutation"));
    }
    bytes.drain(offset..end);
    Ok(())
}

fn insert(
    bytes: &mut Vec<u8>,
    offset: usize,
    span_length: usize,
    parameter: &str,
) -> Result<(), io::Error> {
    if span_length != 0 {
        return Err(invalid_corpus("insert mutation has a nonzero span"));
    }
    if offset > bytes.len() {
        return Err(invalid_corpus("insert offset is out of bounds"));
    }
    let replacement = decode_hex(parameter)?;
    bytes.splice(offset..offset, replacement);
    Ok(())
}

const fn text_error_code(error: LayoutIdTextParseError) -> &'static str {
    match error {
        LayoutIdTextParseError::InputTooLong { .. } => "layout-id.input-too-long",
        LayoutIdTextParseError::MalformedStructure => "layout-id.malformed-structure",
        LayoutIdTextParseError::TrailingData => "layout-id.trailing-data",
        LayoutIdTextParseError::InvalidScheme => "layout-id.wrong-scheme",
        LayoutIdTextParseError::InvalidKind => "layout-id.wrong-kind",
        LayoutIdTextParseError::MalformedVersion => "layout-id.malformed-version",
        LayoutIdTextParseError::UnsupportedVersion { .. } => "layout-id.unsupported-version",
        LayoutIdTextParseError::UnsupportedCodec => "layout-id.unsupported-codec",
        LayoutIdTextParseError::UnsupportedAlgorithm => "layout-id.unsupported-algorithm",
        LayoutIdTextParseError::NonCanonicalPlanLength => "layout-id.noncanonical-plan-length",
        LayoutIdTextParseError::PlanLengthOverflow => "layout-id.plan-length-overflow",
        LayoutIdTextParseError::PlanLengthOutOfBounds { .. } => {
            "layout-id.plan-length-out-of-bounds"
        }
        LayoutIdTextParseError::PlanLengthNotCongruent { .. } => {
            "layout-id.plan-length-not-congruent"
        }
        LayoutIdTextParseError::InvalidDigestLength { .. } => "layout-id.invalid-digest-length",
        LayoutIdTextParseError::NonCanonicalDigestCase => "layout-id.noncanonical-digest-case",
        LayoutIdTextParseError::InvalidDigestAlphabet => "layout-id.invalid-digest-alphabet",
    }
}

const fn binary_error_code(error: LayoutIdBinaryParseError) -> &'static str {
    match error {
        LayoutIdBinaryParseError::WrongLength { .. } => "layout-id.wrong-length",
        LayoutIdBinaryParseError::InvalidMagic { .. } => "layout-id.wrong-magic",
        LayoutIdBinaryParseError::UnsupportedVersion { .. } => "layout-id.unsupported-version",
        LayoutIdBinaryParseError::UnsupportedCodec { .. } => "layout-id.unsupported-codec",
        LayoutIdBinaryParseError::PlanLengthOutOfBounds { .. } => {
            "layout-id.plan-length-out-of-bounds"
        }
        LayoutIdBinaryParseError::PlanLengthNotCongruent { .. } => {
            "layout-id.plan-length-not-congruent"
        }
    }
}

const fn mismatch_code(error: LayoutIdMismatch) -> &'static str {
    match error {
        LayoutIdMismatch::PlanLength { .. } => "layout-id.plan-length-mismatch",
        LayoutIdMismatch::Digest { .. } => "layout-id.mismatch",
    }
}
