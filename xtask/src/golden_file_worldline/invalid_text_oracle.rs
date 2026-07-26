use std::collections::BTreeSet;

use super::canonical_value::{EmptyHex, case_name, decoded_hex, unique};
use super::{Corpus, GoldenError};

const MAX_INVALID_TEXT_BYTES: usize = 4_096;
const MAX_TEXT_BYTES: usize = 109;
const INVALID_TEXT_COLUMNS: [&str; 3] = ["case", "input_hex", "expected_outcome"];
const REQUIRED_TEXT_OUTCOMES: [&str; 13] = [
    "keep.identity.input_too_long",
    "keep.identity.malformed_structure",
    "keep.identity.trailing_data",
    "keep.identity.invalid_scheme",
    "keep.identity.invalid_kind",
    "keep.identity.malformed_version",
    "keep.identity.unsupported_version",
    "keep.identity.unsupported_algorithm",
    "keep.identity.noncanonical_length",
    "keep.identity.length_overflow",
    "keep.identity.invalid_digest_length",
    "keep.identity.noncanonical_digest_case",
    "keep.identity.invalid_digest_alphabet",
];

pub(super) fn check(corpus: &Corpus) -> Result<(), GoldenError> {
    let rows = corpus.rows(
        "invalid-text.tsv",
        "# keep.golden-file-worldline.invalid-text/v1",
        &INVALID_TEXT_COLUMNS,
    )?;
    let mut seen = BTreeSet::new();
    let mut outcomes = BTreeSet::new();
    for row in rows {
        let name = case_name(row.field("case")?, "invalid-text.tsv")?;
        unique(name, &mut seen, "invalid-text.tsv")?;
        let encoded = decoded_hex(
            row.field("input_hex")?,
            &format!("{name} input"),
            MAX_INVALID_TEXT_BYTES,
            EmptyHex::Allow,
        )?;
        let observed = text_outcome(&encoded)?;
        if row.field("expected_outcome")? != observed {
            return Err(GoldenError::violation(format!(
                "{name}: expected text outcome does not match {observed:?}"
            )));
        }
        outcomes.insert(observed);
    }
    if REQUIRED_TEXT_OUTCOMES
        .iter()
        .all(|outcome| outcomes.contains(outcome))
    {
        Ok(())
    } else {
        Err(GoldenError::violation(
            "invalid-text.tsv: required v1 outcome coverage is absent",
        ))
    }
}

fn text_outcome(encoded: &[u8]) -> Result<&'static str, GoldenError> {
    std::str::from_utf8(encoded).map_err(|source| {
        GoldenError::violation(format!("invalid-text.tsv: input is not UTF-8: {source}"))
    })?;
    if encoded.len() > MAX_TEXT_BYTES {
        return Ok("keep.identity.input_too_long");
    }
    let fields = encoded.split(|byte| *byte == b':').collect::<Vec<_>>();
    if fields.len() < 6 || fields.iter().take(6).any(|field| field.is_empty()) {
        return Ok("keep.identity.malformed_structure");
    }
    if fields.len() > 6 {
        return Ok("keep.identity.trailing_data");
    }
    classify_fields(&fields)
}

fn classify_fields(fields: &[&[u8]]) -> Result<&'static str, GoldenError> {
    let [scheme, kind, version, algorithm, length, identity_digest] = fields else {
        return Ok("keep.identity.malformed_structure");
    };
    if *scheme != b"keep" {
        return Ok("keep.identity.invalid_scheme");
    }
    if *kind != b"blob" {
        return Ok("keep.identity.invalid_kind");
    }
    if *version != b"v1" {
        return Ok(version_outcome(version));
    }
    if *algorithm != b"blake3-256" {
        return Ok("keep.identity.unsupported_algorithm");
    }
    if !canonical_decimal_bytes(length) {
        return Ok("keep.identity.noncanonical_length");
    }
    if decimal_bytes_exceed(length, b"18446744073709551615") {
        return Ok("keep.identity.length_overflow");
    }
    classify_digest(identity_digest)
}

fn version_outcome(version: &[u8]) -> &'static str {
    let Some(number) = version.strip_prefix(b"v") else {
        return "keep.identity.malformed_version";
    };
    if canonical_decimal_bytes(number) && !decimal_bytes_exceed(number, b"65535") {
        "keep.identity.unsupported_version"
    } else {
        "keep.identity.malformed_version"
    }
}

fn classify_digest(identity_digest: &[u8]) -> Result<&'static str, GoldenError> {
    if identity_digest.len() != 64 {
        return Ok("keep.identity.invalid_digest_length");
    }
    for character in identity_digest {
        if character.is_ascii_uppercase() && matches!(character, b'A'..=b'F') {
            return Ok("keep.identity.noncanonical_digest_case");
        }
        if !character.is_ascii_digit() && !matches!(character, b'a'..=b'f') {
            return Ok("keep.identity.invalid_digest_alphabet");
        }
    }
    Err(GoldenError::violation(
        "invalid-text.tsv: invalid case unexpectedly parsed",
    ))
}

fn canonical_decimal_bytes(value: &[u8]) -> bool {
    value == b"0"
        || (!value.is_empty()
            && value
                .first()
                .is_some_and(|byte| matches!(byte, b'1'..=b'9'))
            && value.iter().all(u8::is_ascii_digit))
}

fn decimal_bytes_exceed(value: &[u8], maximum: &[u8]) -> bool {
    value.len() > maximum.len() || (value.len() == maximum.len() && value > maximum)
}
