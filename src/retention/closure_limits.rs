//! This module owns one fully admitted retention closure resource policy.

use std::num::{NonZeroU16, NonZeroU64};

use super::{RetentionClosureLimit, RetentionClosureLimitError};

/// Positive closure limits bounded by the version-2 implementation ceilings.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionClosureLimits {
    nodes: NonZeroU64,
    depth: NonZeroU16,
    encoded_bytes: NonZeroU64,
    physical_bytes: NonZeroU64,
}

impl RetentionClosureLimits {
    /// Maximum admitted closure node count.
    pub const MAXIMUM_NODES: u64 = 1_048_576;
    /// Maximum admitted traversal depth.
    pub const MAXIMUM_DEPTH: u16 = 8;
    /// Maximum admitted encoded bytes.
    pub const MAXIMUM_ENCODED_BYTES: u64 = 16_777_216;
    /// Maximum admitted physical bytes.
    pub const MAXIMUM_PHYSICAL_BYTES: u64 = 1_073_741_824;

    /// Admits one complete positive, ceiling-bounded policy.
    ///
    /// # Errors
    ///
    /// Returns the first zero or above-maximum limit in argument order.
    pub fn new(
        nodes: u64,
        depth: u16,
        encoded_bytes: u64,
        physical_bytes: u64,
    ) -> Result<Self, RetentionClosureLimitError> {
        let nodes = admit_u64(RetentionClosureLimit::Nodes, nodes, Self::MAXIMUM_NODES)?;
        let depth = admit_depth(depth)?;
        let encoded_bytes = admit_u64(
            RetentionClosureLimit::EncodedBytes,
            encoded_bytes,
            Self::MAXIMUM_ENCODED_BYTES,
        )?;
        let physical_bytes = admit_u64(
            RetentionClosureLimit::PhysicalBytes,
            physical_bytes,
            Self::MAXIMUM_PHYSICAL_BYTES,
        )?;
        Ok(Self {
            nodes,
            depth,
            encoded_bytes,
            physical_bytes,
        })
    }

    /// Returns the positive closure node limit.
    #[must_use]
    pub const fn nodes(self) -> u64 {
        self.nodes.get()
    }

    /// Returns the positive closure depth limit.
    #[must_use]
    pub const fn depth(self) -> u16 {
        self.depth.get()
    }

    /// Returns the positive encoded-byte limit.
    #[must_use]
    pub const fn encoded_bytes(self) -> u64 {
        self.encoded_bytes.get()
    }

    /// Returns the positive physical-byte limit.
    #[must_use]
    pub const fn physical_bytes(self) -> u64 {
        self.physical_bytes.get()
    }
}

fn admit_u64(
    limit: RetentionClosureLimit,
    observed: u64,
    maximum: u64,
) -> Result<NonZeroU64, RetentionClosureLimitError> {
    let value = NonZeroU64::new(observed).ok_or(RetentionClosureLimitError::Zero { limit })?;
    if observed > maximum {
        return Err(RetentionClosureLimitError::AboveMaximum {
            limit,
            maximum,
            observed,
        });
    }
    Ok(value)
}

fn admit_depth(observed: u16) -> Result<NonZeroU16, RetentionClosureLimitError> {
    let limit = RetentionClosureLimit::Depth;
    let value = NonZeroU16::new(observed).ok_or(RetentionClosureLimitError::Zero { limit })?;
    if observed > RetentionClosureLimits::MAXIMUM_DEPTH {
        return Err(RetentionClosureLimitError::AboveMaximum {
            limit,
            maximum: u64::from(RetentionClosureLimits::MAXIMUM_DEPTH),
            observed: u64::from(observed),
        });
    }
    Ok(value)
}
