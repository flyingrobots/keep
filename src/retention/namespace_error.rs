//! This module owns typed retention namespace admission failures.

use std::error::Error;
use std::fmt;

/// Failure to admit opaque retention namespace bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionNamespaceError {
    /// The namespace was empty.
    Empty,
    /// The namespace exceeded the version-2 byte ceiling.
    TooLong {
        /// Maximum admitted length.
        maximum: u8,
        /// Observed byte length.
        observed: usize,
    },
}

impl fmt::Display for RetentionNamespaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("retention namespace must not be empty"),
            Self::TooLong { maximum, observed } => write!(
                formatter,
                "retention namespace has {observed} bytes; maximum is {maximum}"
            ),
        }
    }
}

impl Error for RetentionNamespaceError {}
