//! This module owns semantic refusals when a manifest selects a retained root.

use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

use crate::{RetentionRootDigest, RootGeneration};

/// Why a selected root's size or decoded selection cannot be admitted.
#[derive(Debug)]
#[non_exhaustive]
pub enum RetentionSnapshotRefusal {
    /// The filesystem length cannot be represented in this address space.
    LengthAddressSpace {
        /// The observed file length.
        observed: u64,
        /// The exact conversion failure.
        source: TryFromIntError,
    },
    /// The root exceeds the protocol's allocation bound.
    LengthBound {
        /// The maximum encoded length admitted by the root format.
        maximum: usize,
        /// The filesystem length observed before allocation.
        observed: usize,
    },
    /// A selected record disappeared between observation and exact reading.
    SelectedRootAbsent {
        /// The generation the manifest selects.
        expected_generation: RootGeneration,
        /// The digest the manifest selects.
        expected_digest: RetentionRootDigest,
    },
    /// The decoded root is not the manifest's exact selection.
    SelectionMismatch {
        /// The manifest-selected generation.
        expected_generation: RootGeneration,
        /// The decoded generation.
        observed_generation: RootGeneration,
        /// The manifest-selected digest.
        expected_digest: RetentionRootDigest,
        /// The decoded digest.
        observed_digest: RetentionRootDigest,
    },
}

impl fmt::Display for RetentionSnapshotRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthAddressSpace { observed, .. } => {
                write!(
                    formatter,
                    "selected root length {observed} exceeds the address space"
                )
            }
            Self::LengthBound { maximum, observed } => {
                write!(
                    formatter,
                    "selected root length {observed} exceeds format bound {maximum}"
                )
            }
            Self::SelectedRootAbsent {
                expected_generation,
                ..
            } => {
                write!(
                    formatter,
                    "selected root generation {expected_generation:?} is absent"
                )
            }
            Self::SelectionMismatch { .. } => formatter
                .write_str("selected root disagrees with the manifest's generation or digest"),
        }
    }
}

impl Error for RetentionSnapshotRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LengthAddressSpace { source, .. } => Some(source),
            Self::LengthBound { .. }
            | Self::SelectedRootAbsent { .. }
            | Self::SelectionMismatch { .. } => None,
        }
    }
}
