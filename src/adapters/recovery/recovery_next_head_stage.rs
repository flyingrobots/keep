//! This module owns admitted recovery states for `head.next`.

use super::ChecksummedPublicationHead;

/// Semantic state of complete caller-supplied `head.next` bytes.
#[must_use]
pub enum RecoveryNextHeadStage<'a> {
    /// The fixed-width publication head is incomplete.
    Truncated {
        /// Required publication-head byte count.
        required: usize,
        /// Supplied byte count.
        observed: usize,
    },
    /// Fully framing- and checksum-verified publication head.
    Complete(ChecksummedPublicationHead<'a>),
}
