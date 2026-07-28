//! This module owns substitution-free fuzz policy syntax and scalar admission.

use std::collections::BTreeMap;

use super::PolicyError;

const EXPECTED_KEYS: [&str; 13] = [
    "CARGO_FUZZ_VERSION",
    "FUZZ_CMIN_SECONDS_PER_TARGET",
    "FUZZ_CORPUS_MAX_BYTES",
    "FUZZ_CORPUS_MAX_FILES",
    "FUZZ_CORPUS_RETENTION_DAYS",
    "FUZZ_INPUT_TIMEOUT_SECONDS",
    "FUZZ_MAX_INPUT_BYTES",
    "FUZZ_RSS_LIMIT_MB",
    "FUZZ_SCHEDULED_FAILURE_RETENTION_DAYS",
    "FUZZ_SCHEDULED_SECONDS_PER_TARGET",
    "FUZZ_SMOKE_FAILURE_RETENTION_DAYS",
    "FUZZ_SMOKE_SECONDS_PER_TARGET",
    "FUZZ_TOOLCHAIN",
];

pub(super) fn parse_assignments(raw: &str) -> Result<BTreeMap<&str, &str>, PolicyError> {
    let mut values = BTreeMap::new();
    for (index, raw_line) in raw.lines().enumerate() {
        let line_number = index
            .checked_add(1)
            .ok_or(PolicyError::Line { line: index })?;
        admit_line(&mut values, raw_line, line_number)?;
    }
    let missing = EXPECTED_KEYS
        .into_iter()
        .filter(|key| !values.contains_key(key))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(values)
    } else {
        Err(PolicyError::Missing(missing))
    }
}

fn admit_line<'a>(
    values: &mut BTreeMap<&'a str, &'a str>,
    raw_line: &'a str,
    line_number: usize,
) -> Result<(), PolicyError> {
    let line = raw_line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(());
    }
    let Some((key, value)) = line.split_once('=') else {
        return Err(PolicyError::Line { line: line_number });
    };
    if value.contains('=') {
        return Err(PolicyError::Line { line: line_number });
    }
    if !EXPECTED_KEYS.contains(&key) || value.is_empty() || values.contains_key(key) {
        return Err(PolicyError::Key { line: line_number });
    }
    if value.chars().any(char::is_whitespace) {
        return Err(PolicyError::Whitespace { line: line_number });
    }
    values.insert(key, value);
    Ok(())
}

pub(super) fn value<'a>(
    values: &'a BTreeMap<&str, &'a str>,
    key: &'static str,
) -> Result<&'a str, PolicyError> {
    values
        .get(key)
        .copied()
        .ok_or_else(|| PolicyError::Missing(vec![key]))
}

pub(super) fn bounded(
    values: &BTreeMap<&str, &str>,
    key: &'static str,
    minimum: u64,
    maximum: u64,
) -> Result<u64, PolicyError> {
    let raw = value(values, key)?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(PolicyError::InvalidInteger(key));
    }
    let parsed = raw
        .parse::<u64>()
        .map_err(|_source| PolicyError::InvalidInteger(key))?;
    if (minimum..=maximum).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(PolicyError::Bound {
            key,
            minimum,
            maximum,
        })
    }
}

pub(super) fn is_exact_version(value: &str) -> bool {
    let mut components = value.split('.');
    let valid = (0..3).all(|_| {
        components
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    });
    valid && components.next().is_none()
}

pub(super) fn is_dated_nightly(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 18
        && bytes.get(..8) == Some(b"nightly-")
        && bytes.get(8..12).is_some_and(is_ascii_digits)
        && bytes.get(12) == Some(&b'-')
        && bytes.get(13..15).is_some_and(is_ascii_digits)
        && bytes.get(15) == Some(&b'-')
        && bytes.get(16..18).is_some_and(is_ascii_digits)
}

fn is_ascii_digits(bytes: &[u8]) -> bool {
    bytes.iter().all(u8::is_ascii_digit)
}
