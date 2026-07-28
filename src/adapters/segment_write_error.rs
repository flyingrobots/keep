//! Typed staged-segment writing and sealing failures.

use std::collections::TryReserveError;
use std::io;

use super::{SegmentDurabilityPhase, SegmentRecordIdentity, SegmentSealError, SegmentWritePhase};

/// A staged segment could not advance through an exact write or durability
/// transition.
#[derive(Debug)]
pub enum SegmentWriteError {
    /// The next record would exceed the configured count cap.
    RecordCountLimit {
        /// Configured record count cap.
        maximum: u32,
        /// Attempted record count.
        observed: u32,
    },
    /// The record count could not advance.
    RecordCountArithmetic {
        /// Current record count.
        observed: u32,
    },
    /// The staged segment already contains the logical identity.
    DuplicateRecordIdentity {
        /// Repeated logical identity.
        identity: SegmentRecordIdentity,
    },
    /// Memory for the bounded duplicate index could not be reserved.
    IdentityIndexAllocation {
        /// Logical identity being prepared.
        identity: SegmentRecordIdentity,
        /// Allocation reservation failure.
        source: TryReserveError,
    },
    /// The next record would overflow the physical prefix length.
    SegmentLengthArithmetic {
        /// Current pre-seal byte count.
        bytes_before_record: u64,
        /// Attempted complete record byte count.
        record_length: u64,
    },
    /// The next record would exceed the immutable segment length bound.
    SegmentLengthLimit {
        /// Immutable complete segment byte cap.
        maximum: u64,
        /// Attempted complete segment byte count.
        observed: u64,
    },
    /// A successful write reported more bytes than were supplied.
    InvalidWriteCount {
        /// Exact write boundary.
        phase: SegmentWritePhase,
        /// Largest lawful count for this call.
        maximum: usize,
        /// Count reported by the stage.
        observed: usize,
        /// Bytes successfully staged before the invalid report.
        bytes_written: u64,
    },
    /// A nonempty write made no progress.
    WriteZero {
        /// Exact write boundary.
        phase: SegmentWritePhase,
        /// Bytes successfully staged before the zero write.
        bytes_written: u64,
    },
    /// A successful host-width write count could not advance the wire offset.
    WriteLengthArithmetic {
        /// Exact write boundary.
        phase: SegmentWritePhase,
        /// Bytes successfully staged before the failed transition.
        bytes_written: u64,
        /// Host-width count that could not be represented or added.
        incoming: usize,
    },
    /// The stage returned a non-interruption write failure.
    Write {
        /// Exact write boundary.
        phase: SegmentWritePhase,
        /// Bytes successfully staged before the failure.
        bytes_written: u64,
        /// Underlying stage failure.
        source: io::Error,
    },
    /// Flushing a durability boundary failed.
    Flush {
        /// Exact durability boundary.
        phase: SegmentDurabilityPhase,
        /// Underlying flush failure.
        source: io::Error,
    },
    /// Synchronizing a durability boundary failed.
    Synchronize {
        /// Exact durability boundary.
        phase: SegmentDurabilityPhase,
        /// Underlying synchronization failure.
        source: io::Error,
    },
    /// Constructing the exact terminal seal failed.
    Seal {
        /// Exact seal construction failure.
        source: SegmentSealError,
    },
}
