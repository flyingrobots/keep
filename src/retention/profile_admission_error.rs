//! This module owns typed retention-profile admission failures.

use std::error::Error;
use std::fmt;

/// Failure to admit a retention realization-profile coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionProfileAdmissionError {
    /// The identity and version pair is not registered.
    UnsupportedCoordinate {
        /// Registered identity expected by this Keep version.
        expected_identity: u32,
        /// Registered version expected by this Keep version.
        expected_version: u32,
        /// Identity observed at the boundary.
        observed_identity: u32,
        /// Version observed at the boundary.
        observed_version: u32,
    },
    /// The registered coordinate carried different definition bytes.
    DefinitionDigestMismatch {
        /// Exact registered definition digest.
        expected: [u8; 32],
        /// Digest observed at the boundary.
        observed: [u8; 32],
    },
}

impl fmt::Display for RetentionProfileAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCoordinate {
                expected_identity,
                expected_version,
                observed_identity,
                observed_version,
            } => write!(
                formatter,
                "unsupported retention profile {observed_identity}/{observed_version}; \
                 expected {expected_identity}/{expected_version}"
            ),
            Self::DefinitionDigestMismatch { expected, observed } => write!(
                formatter,
                "retention profile definition digest mismatch: expected {expected:02x?}, \
                 observed {observed:02x?}"
            ),
        }
    }
}

impl Error for RetentionProfileAdmissionError {}
