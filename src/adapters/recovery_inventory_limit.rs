//! This module owns the recovery inventory entry-count bound.

use std::error::Error;
use std::fmt;

/// Maximum entries retained by one complete recovery inventory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryInventoryLimit {
    maximum: u64,
}

impl RecoveryInventoryLimit {
    /// Protocol-wide maximum inventory entry count.
    pub const PROTOCOL_MAXIMUM: u64 = 2_097_152;

    /// Admits a configured maximum no greater than the protocol ceiling.
    ///
    /// Zero is valid and admits only an empty inventory.
    ///
    /// # Errors
    ///
    /// Returns [`RecoveryInventoryLimitError`] when `maximum` exceeds the
    /// protocol ceiling.
    pub const fn new(maximum: u64) -> Result<Self, RecoveryInventoryLimitError> {
        if maximum <= Self::PROTOCOL_MAXIMUM {
            Ok(Self { maximum })
        } else {
            Err(RecoveryInventoryLimitError::AboveProtocolMaximum {
                requested: maximum,
                maximum: Self::PROTOCOL_MAXIMUM,
            })
        }
    }

    /// Returns the protocol-wide maximum.
    #[must_use]
    pub const fn protocol_maximum() -> Self {
        Self {
            maximum: Self::PROTOCOL_MAXIMUM,
        }
    }

    /// Returns the admitted maximum.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.maximum
    }
}

/// Why a recovery inventory limit is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryInventoryLimitError {
    /// The requested limit exceeds the protocol ceiling.
    AboveProtocolMaximum {
        /// Caller-requested limit.
        requested: u64,
        /// Protocol maximum.
        maximum: u64,
    },
}

impl fmt::Display for RecoveryInventoryLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AboveProtocolMaximum { requested, maximum } => write!(
                formatter,
                "recovery inventory limit {requested} exceeds protocol maximum {maximum}"
            ),
        }
    }
}

impl Error for RecoveryInventoryLimitError {}
