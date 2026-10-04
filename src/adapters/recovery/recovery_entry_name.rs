//! This module owns one bounded-inventory path-component spelling.

use std::error::Error;
use std::fmt;

/// One raw path-component name observed during recovery inventory.
///
/// The value may be a noncanonical protocol name so later classification can
/// report ambiguity. Construction rejects only spellings that cannot identify
/// one child entry. The owned allocation is bounded by the admitted filesystem
/// component and the inventory entry-count limit.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RecoveryEntryName {
    bytes: Vec<u8>,
}

impl RecoveryEntryName {
    /// Admits one raw, nonempty child-entry spelling.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryEntryNameError`] for an empty name, NUL, path
    /// separator, or dot component.
    pub fn new(bytes: Vec<u8>) -> Result<Self, RecoveryEntryNameError> {
        if bytes.is_empty() {
            return Err(RecoveryEntryNameError::Empty);
        }
        if bytes.contains(&0) {
            return Err(RecoveryEntryNameError::Nul);
        }
        if bytes.contains(&b'/') {
            return Err(RecoveryEntryNameError::PathSeparator);
        }
        if bytes == b"." || bytes == b".." {
            return Err(RecoveryEntryNameError::DotComponent);
        }
        Ok(Self { bytes })
    }

    /// Returns the exact raw name bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Why a recovery entry name cannot identify one child entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryEntryNameError {
    /// The name is empty.
    Empty,
    /// The name contains NUL.
    Nul,
    /// The name contains a path separator.
    PathSeparator,
    /// The name is `.` or `..`.
    DotComponent,
}

impl fmt::Display for RecoveryEntryNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "recovery entry name is empty",
            Self::Nul => "recovery entry name contains NUL",
            Self::PathSeparator => "recovery entry name contains a path separator",
            Self::DotComponent => "recovery entry name is a dot component",
        })
    }
}

impl Error for RecoveryEntryNameError {}
