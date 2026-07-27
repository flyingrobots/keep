//! Bounded, platform-neutral admission for repository protocol scalars.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::string::FromUtf8Error;

/// Whether an empty hexadecimal scalar is part of a field's grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmptyHex {
    /// Admit the empty scalar.
    Allow,
    /// Refuse the empty scalar.
    Refuse,
}

/// A refusal produced while admitting final-LF-framed UTF-8 lines.
#[derive(Debug)]
pub enum FramedLinesError {
    /// The input exceeded the caller's explicit byte bound.
    ExceedsMaximum {
        /// Maximum admitted byte length.
        maximum: usize,
    },
    /// The input was empty, lacked a final LF, or contained CR bytes.
    FinalLfOnly,
    /// The admitted bytes were not UTF-8.
    Utf8(FromUtf8Error),
    /// The framed protocol contained an empty line.
    BlankLine,
}

/// A refusal produced while decoding canonical lowercase hexadecimal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HexError {
    /// The field grammar refuses an empty scalar.
    Empty,
    /// The maximum byte bound could not be converted to a digit bound.
    BoundOverflow,
    /// The value was odd-length or exceeded its byte bound.
    InvalidLength,
    /// The value contained a byte outside lowercase hexadecimal.
    NonCanonicalAlphabet,
    /// A decoded hexadecimal byte overflowed.
    ByteOverflow,
}

/// A refusal produced when a tab-separated row has the wrong arity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldCountError;

/// A refusal produced by the platform-neutral relative path grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelativePathError {
    /// The path is empty.
    Empty,
    /// The path begins with a POSIX root separator.
    Absolute,
    /// The path contains a backslash.
    Backslash,
    /// The path contains a colon.
    Colon,
    /// The path contains a NUL.
    Nul,
    /// The path contains an empty segment.
    EmptySegment,
    /// The path contains a current-directory segment.
    DotSegment,
    /// The path contains a parent-directory segment.
    ParentSegment,
}

/// Named canonical profile for Golden File Worldline source paths.
pub const POSIX_RELATIVE_PATH_PROFILE: &str = "keep.golden-file-worldline.path/v1";

/// Admit bounded UTF-8 lines under final-LF-only framing.
///
/// # Errors
///
/// Returns [`FramedLinesError`] when the byte bound, framing, UTF-8, or
/// nonblank-line invariant is violated.
pub fn framed_lines(input: &[u8], maximum: usize) -> Result<Vec<String>, FramedLinesError> {
    if input.len() > maximum {
        return Err(FramedLinesError::ExceedsMaximum { maximum });
    }
    if input.is_empty() || !input.ends_with(b"\n") || input.contains(&b'\r') {
        return Err(FramedLinesError::FinalLfOnly);
    }
    let text = String::from_utf8(input.to_vec()).map_err(FramedLinesError::Utf8)?;
    let framed = text
        .strip_suffix('\n')
        .ok_or(FramedLinesError::FinalLfOnly)?;
    let lines = framed.split('\n').map(str::to_owned).collect::<Vec<_>>();
    if lines.iter().any(String::is_empty) {
        Err(FramedLinesError::BlankLine)
    } else {
        Ok(lines)
    }
}

/// Split one tab-separated row under an exact field-count invariant.
///
/// # Errors
///
/// Returns [`FieldCountError`] when the observed field count differs from
/// `expected`.
pub fn tab_fields(line: &str, expected: usize) -> Result<Vec<&str>, FieldCountError> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() == expected {
        Ok(fields)
    } else {
        Err(FieldCountError)
    }
}

/// Decode a bounded canonical lowercase hexadecimal scalar.
///
/// # Errors
///
/// Returns [`HexError`] when the value violates its empty-value policy, byte
/// bound, even-width invariant, lowercase alphabet, or checked arithmetic.
pub fn decode_lower_hex(
    value: &str,
    maximum_bytes: usize,
    empty: EmptyHex,
) -> Result<Vec<u8>, HexError> {
    if value.is_empty() && empty == EmptyHex::Refuse {
        return Err(HexError::Empty);
    }
    let maximum_digits = maximum_bytes
        .checked_mul(2)
        .ok_or(HexError::BoundOverflow)?;
    if value.len() > maximum_digits || !value.len().is_multiple_of(2) {
        return Err(HexError::InvalidLength);
    }
    value.as_bytes().chunks_exact(2).map(decode_pair).collect()
}

/// Admit a relative path under [`POSIX_RELATIVE_PATH_PROFILE`].
///
/// # Errors
///
/// Returns [`RelativePathError`] for empty, absolute, colon-containing,
/// backslash-separated, NUL-containing, dot-segment, or empty-segment paths.
pub fn posix_relative_path(parameter: &str) -> Result<PathBuf, RelativePathError> {
    if parameter.is_empty() {
        return Err(RelativePathError::Empty);
    }
    if parameter.starts_with('/') {
        return Err(RelativePathError::Absolute);
    }
    if parameter.contains('\\') {
        return Err(RelativePathError::Backslash);
    }
    if parameter.contains(':') {
        return Err(RelativePathError::Colon);
    }
    if parameter.contains('\0') {
        return Err(RelativePathError::Nul);
    }
    let mut relative = PathBuf::new();
    for segment in parameter.split('/') {
        match segment {
            "" => return Err(RelativePathError::EmptySegment),
            "." => return Err(RelativePathError::DotSegment),
            ".." => return Err(RelativePathError::ParentSegment),
            _ => relative.push(segment),
        }
    }
    Ok(relative)
}

fn decode_pair(pair: &[u8]) -> Result<u8, HexError> {
    let [high_byte, low_byte] = pair else {
        return Err(HexError::InvalidLength);
    };
    let high = hex_nibble(*high_byte).ok_or(HexError::NonCanonicalAlphabet)?;
    let low = hex_nibble(*low_byte).ok_or(HexError::NonCanonicalAlphabet)?;
    high.checked_mul(16)
        .and_then(|shifted| shifted.checked_add(low))
        .ok_or(HexError::ByteOverflow)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|offset| offset.checked_add(10)),
        _ => None,
    }
}

impl fmt::Display for FramedLinesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExceedsMaximum { maximum } => {
                write!(formatter, "input exceeds {maximum} bytes")
            }
            Self::FinalLfOnly => formatter.write_str("input is not final-LF-only"),
            Self::Utf8(_) => formatter.write_str("input is not UTF-8"),
            Self::BlankLine => formatter.write_str("input contains a blank line"),
        }
    }
}

impl Error for FramedLinesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Utf8(source) => Some(source),
            Self::ExceedsMaximum { .. } | Self::FinalLfOnly | Self::BlankLine => None,
        }
    }
}

impl fmt::Display for HexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for HexError {}

impl fmt::Display for FieldCountError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("field count differs from the declared schema")
    }
}

impl Error for FieldCountError {}

impl fmt::Display for RelativePathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "path is empty",
            Self::Absolute => "path is absolute",
            Self::Backslash => "path contains a backslash",
            Self::Colon => "path contains a colon",
            Self::Nul => "path contains a NUL",
            Self::EmptySegment => "path contains an empty segment",
            Self::DotSegment => "path contains a current-directory segment",
            Self::ParentSegment => "path contains a parent-directory segment",
        })
    }
}

impl Error for RelativePathError {}
