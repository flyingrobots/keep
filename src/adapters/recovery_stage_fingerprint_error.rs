//! This module owns recovery-stage fingerprinting failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::RecoveryStage;

/// Why one fixed recovery stage could not produce bounded exact evidence.
#[derive(Debug)]
pub enum RecoveryStageFingerprintError {
    /// The stream produced at least one byte above its maximum.
    EvidenceOversized {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Name-selected maximum.
        maximum: u64,
        /// Bounded lower limit on the observed byte count.
        observed_at_least: u64,
    },
    /// A reader reported more bytes than it was offered.
    ReaderContract {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Exact stream offset before the call.
        offset: u64,
        /// Buffer bytes offered to the reader.
        offered: usize,
        /// Byte count reported by the reader.
        observed: usize,
    },
    /// A bounded platform length could not be represented.
    PlatformLength {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Bounded value that could not be represented.
        observed: u64,
    },
    /// The fixed stack buffer width could not be represented as `u64`.
    PlatformBufferLength {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Stack-buffer byte width.
        observed: usize,
    },
    /// Stream offset arithmetic overflowed.
    LengthOverflow {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Offset before the overflowing addition.
        offset: u64,
        /// Reported increment.
        increment: u64,
    },
    /// The underlying reader failed.
    Read {
        /// Fixed stage being observed.
        stage: RecoveryStage,
        /// Exact stream offset before the failed call.
        offset: u64,
        /// Underlying failure.
        source: io::Error,
    },
}

impl fmt::Display for RecoveryStageFingerprintError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvidenceOversized {
                stage,
                maximum,
                observed_at_least,
            } => write!(
                formatter,
                "{stage} produced at least {observed_at_least} bytes above maximum {maximum}"
            ),
            Self::ReaderContract {
                stage,
                offset,
                offered,
                observed,
            } => write!(
                formatter,
                "{stage} reader reported {observed} bytes from {offered} offered at offset {offset}"
            ),
            Self::PlatformLength { stage, observed } => write!(
                formatter,
                "{stage} bounded read length {observed} does not fit this platform"
            ),
            Self::PlatformBufferLength { stage, observed } => write!(
                formatter,
                "{stage} buffer length {observed} does not fit the stream coordinate"
            ),
            Self::LengthOverflow {
                stage,
                offset,
                increment,
            } => write!(
                formatter,
                "{stage} stream offset {offset} overflows with increment {increment}"
            ),
            Self::Read {
                stage,
                offset,
                source,
            } => write!(
                formatter,
                "{stage} read failed at offset {offset}: {source}"
            ),
        }
    }
}

impl Error for RecoveryStageFingerprintError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::EvidenceOversized { .. }
            | Self::ReaderContract { .. }
            | Self::PlatformLength { .. }
            | Self::PlatformBufferLength { .. }
            | Self::LengthOverflow { .. } => None,
        }
    }
}
