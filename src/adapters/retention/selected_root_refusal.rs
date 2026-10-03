//! This module owns exact contradictions in manifest-selected root admission.

use crate::{RetentionNamespaceDigest, RetentionRootDigest, RootGeneration};
use std::error::Error;
use std::fmt;

/// Why present root evidence does not satisfy its manifest selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RetentionSelectedRootRefusal {
    /// The selected name disappeared between observation and open.
    Absent,
    /// The file length exceeds the root format's byte ceiling.
    Length {
        /// Maximum canonical record bytes.
        maximum: u64,
        /// Observed file bytes.
        observed: u64,
    },
    /// A filesystem length cannot be represented on this host.
    HostLength {
        /// Observed file bytes.
        observed: u64,
    },
    /// The decoded record belongs to a different namespace.
    Namespace {
        /// Namespace selected by the manifest lookup.
        expected: RetentionNamespaceDigest,
        /// Namespace authenticated from the record.
        observed: RetentionNamespaceDigest,
    },
    /// The canonical root does not match the selected generation and digest.
    Coordinate {
        /// Selected root generation.
        expected_generation: RootGeneration,
        /// Decoded root generation.
        observed_generation: RootGeneration,
        /// Selected root digest.
        expected_digest: RetentionRootDigest,
        /// Decoded root digest.
        observed_digest: RetentionRootDigest,
    },
}

impl fmt::Display for RetentionSelectedRootRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Absent => "selected root is absent",
            Self::Length { .. } => "selected root exceeds the format bound",
            Self::HostLength { .. } => "root length overflow",
            Self::Namespace { .. } => "selected root namespace disagrees",
            Self::Coordinate { .. } => "selected root does not decode to the manifest's selection",
        })
    }
}
impl Error for RetentionSelectedRootRefusal {}
