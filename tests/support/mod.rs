//! Shared deterministic integration-corpus support.

mod byte_readers;
mod byte_writers;

use std::io;

use keep::{ChunkSpan, FastCdc};

pub use byte_readers::{FailingReader, LyingReader, PartitionReader};
pub use byte_writers::{FailingWriter, LyingWriter, PartitionWriter, ZeroWriter};

const LAYOUTS: &str = include_str!("../../conformance/layout/v1/layouts.tsv");

/// Returns one required tab-separated corpus field.
///
/// # Errors
///
/// Returns a corpus error when the row has no field at `index`.
pub fn field(row: &str, index: usize) -> Result<&str, io::Error> {
    field_unchecked(row, index).ok_or_else(|| invalid_corpus("TSV row is missing a field"))
}

/// Returns one tab-separated corpus field when present.
#[must_use]
pub fn field_unchecked(row: &str, index: usize) -> Option<&str> {
    row.split('\t').nth(index)
}

/// Returns one required field from a named frozen layout case.
///
/// # Errors
///
/// Returns a corpus error when the case or requested field is missing.
pub fn layout_case_field(case: &str, index: usize) -> Result<&'static str, io::Error> {
    let row = LAYOUTS
        .lines()
        .skip(2)
        .find(|row| field_unchecked(row, 0) == Some(case))
        .ok_or_else(|| invalid_corpus("layout case is missing"))?;
    field(row, index)
}

/// Returns the lowercase-hex record fixture for a named layout case.
///
/// # Errors
///
/// Returns a corpus error when the case has no frozen record.
pub fn layout_record_fixture(case: &str) -> Result<&'static str, io::Error> {
    match case {
        "empty" => Ok(include_str!("../../conformance/layout/v1/empty.layout.hex").trim_end()),
        "one-zero" => {
            Ok(include_str!("../../conformance/layout/v1/one-zero.layout.hex").trim_end())
        }
        "max-plus-one-zeros" => Ok(include_str!(
            "../../conformance/layout/v1/max-plus-one-zeros.layout.hex"
        )
        .trim_end()),
        "zeros-long" => {
            Ok(include_str!("../../conformance/layout/v1/zeros-long.layout.hex").trim_end())
        }
        _ => Err(invalid_corpus("unknown layout record fixture")),
    }
}

/// Decodes the exact binary record fixture for a named layout case.
///
/// # Errors
///
/// Returns a corpus error for an unknown case or malformed hexadecimal fixture.
pub fn layout_record_bytes(case: &str) -> Result<Vec<u8>, io::Error> {
    decode_hex(layout_record_fixture(case)?)
}

/// Materializes the deterministic source recipe for a frozen layout case.
///
/// # Errors
///
/// Returns a corpus error for an unknown case, malformed length, or failed
/// bounded allocation.
pub fn layout_source_bytes(case: &str, count: &str) -> Result<Vec<u8>, io::Error> {
    if !matches!(
        case,
        "empty" | "one-zero" | "max-plus-one-zeros" | "zeros-long"
    ) {
        return Err(invalid_corpus("unknown layout source recipe"));
    }
    let length = count
        .parse::<usize>()
        .map_err(|_source| invalid_corpus("layout source length is not usize"))?;
    let mut source = Vec::new();
    source
        .try_reserve_exact(length)
        .map_err(|_source| invalid_corpus("layout source allocation failed"))?;
    source.resize(length, 0_u8);
    Ok(source)
}

/// Runs the registered detector and returns its exact ordered spans.
///
/// # Errors
///
/// Returns the detector's typed failure.
pub fn detect_spans(bytes: &[u8]) -> Result<Vec<ChunkSpan>, keep::ChunkingError> {
    let mut spans = Vec::new();
    let mut detector = FastCdc::new();
    detector.feed(bytes, |span| spans.push(span))?;
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    Ok(spans)
}

/// Extracts the expected error from a negative test result.
///
/// # Errors
///
/// Returns a corpus error when the operation unexpectedly succeeds.
pub fn require_error<T, E>(result: Result<T, E>, message: &'static str) -> Result<E, io::Error> {
    match result {
        Ok(_) => Err(invalid_corpus(message)),
        Err(error) => Ok(error),
    }
}

/// Decodes exact lowercase hexadecimal fixture transport.
///
/// # Errors
///
/// Returns an I/O-shaped corpus error for odd width, invalid digits, or
/// impossible checked arithmetic.
pub fn decode_hex(encoded: &str) -> Result<Vec<u8>, io::Error> {
    if !encoded.len().is_multiple_of(2) {
        return Err(invalid_corpus("hex input has odd length"));
    }
    let capacity = encoded
        .len()
        .checked_div(2)
        .ok_or_else(|| invalid_corpus("hexadecimal width divisor is zero"))?;
    let mut decoded = Vec::with_capacity(capacity);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = pair
            .first()
            .copied()
            .and_then(hex_nibble)
            .ok_or_else(|| invalid_corpus("invalid hexadecimal input"))?;
        let low = pair
            .get(1)
            .copied()
            .and_then(hex_nibble)
            .ok_or_else(|| invalid_corpus("invalid hexadecimal input"))?;
        let byte = high
            .checked_shl(4)
            .and_then(|shifted| shifted.checked_add(low))
            .ok_or_else(|| invalid_corpus("hexadecimal byte overflow"))?;
        decoded.push(byte);
    }
    Ok(decoded)
}

/// Constructs a deterministic malformed-corpus failure.
pub fn invalid_corpus(message: &'static str) -> io::Error {
    io::Error::other(message)
}

const fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0' => Some(0),
        b'1' => Some(1),
        b'2' => Some(2),
        b'3' => Some(3),
        b'4' => Some(4),
        b'5' => Some(5),
        b'6' => Some(6),
        b'7' => Some(7),
        b'8' => Some(8),
        b'9' => Some(9),
        b'a' => Some(10),
        b'b' => Some(11),
        b'c' => Some(12),
        b'd' => Some(13),
        b'e' => Some(14),
        b'f' => Some(15),
        _ => None,
    }
}
