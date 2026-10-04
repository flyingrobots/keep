//! This module owns gc filesystem protocol-state refusals.

use std::error::Error;
use std::fmt;

/// A semantic filesystem protocol refusal, preserved through I/O adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FilesystemGcRefusal {
    /// GC intent vanished.
    IntentAbsent,
    /// GC receipt vanished.
    ReceiptAbsent,
    /// No GC retirement is in progress.
    NoRetirement,
    /// A GC candidate is still present.
    CandidateStillPresent,
    /// GC candidate index out of range.
    CandidateIndex,
    /// GC candidate kind or length disagrees with the intent.
    CandidateKindOrLength,
    /// GC receipt is absent before intent removal.
    ReceiptAbsentBeforeIntentRemoval,
    /// GC intent bound overflow.
    IntentBoundOverflow,
}

impl fmt::Display for FilesystemGcRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::IntentAbsent => "GC intent vanished",
            Self::ReceiptAbsent => "GC receipt vanished",
            Self::NoRetirement => "no GC retirement is in progress",
            Self::CandidateStillPresent => "a GC candidate is still present",
            Self::CandidateIndex => "GC candidate index out of range",
            Self::CandidateKindOrLength => "GC candidate kind or length disagrees with the intent",
            Self::ReceiptAbsentBeforeIntentRemoval => "GC receipt is absent before intent removal",
            Self::IntentBoundOverflow => "GC intent bound overflow",
        })
    }
}

impl Error for FilesystemGcRefusal {}
