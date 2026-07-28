//! Typed exclusive segment-stage creation failures.

use std::io;

/// A fixed-name filesystem segment stage could not be created exclusively.
#[derive(Debug)]
pub enum SegmentStageCreateError {
    /// The atomic no-replacement creation operation failed.
    Create {
        /// Underlying filesystem refusal.
        source: io::Error,
    },
}
