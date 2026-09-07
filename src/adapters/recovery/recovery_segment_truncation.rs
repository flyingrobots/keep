//! This module owns exact segment-stage truncation coordinates.

/// Known incomplete boundary in one `current.seg` byte sequence.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoverySegmentTruncation {
    /// The fixed segment header is incomplete.
    Header {
        /// Required fixed-header byte count.
        required: usize,
        /// Supplied byte count.
        observed: usize,
    },
    /// The next tail header is too short to classify as record or seal.
    TailHeader {
        /// Zero-based next record position.
        record_index: u32,
        /// Physical stage byte offset.
        offset: u64,
        /// Required record-header byte count.
        required: usize,
        /// Remaining tail byte count.
        observed: usize,
    },
    /// A valid record header declares more bytes than remain.
    Record {
        /// Zero-based record position.
        record_index: u32,
        /// Physical stage byte offset.
        offset: u64,
        /// Declared complete record byte count.
        expected: u64,
        /// Remaining record byte count.
        observed: usize,
    },
    /// A recognized terminal seal is incomplete.
    Seal {
        /// Physical stage byte offset.
        offset: u64,
        /// Required fixed-seal byte count.
        required: usize,
        /// Remaining seal byte count.
        observed: usize,
    },
}
