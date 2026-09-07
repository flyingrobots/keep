//! This module owns admitted recovery states for one catalog stage.

use super::ChecksummedCatalog;

/// Semantic state of complete caller-supplied `current.cat` bytes.
#[must_use]
pub enum RecoveryCatalogStage<'a> {
    /// The fixed catalog header is incomplete.
    HeaderTruncated {
        /// Required fixed-header byte count.
        required: usize,
        /// Supplied byte count.
        observed: usize,
    },
    /// The admitted header declares more bytes than were supplied.
    BodyTruncated {
        /// Declared canonical catalog byte count.
        expected: u64,
        /// Supplied byte count.
        observed: usize,
    },
    /// Fully framing-, checksum-, digest-, and entry-verified catalog.
    Complete(ChecksummedCatalog<'a>),
}
