//! This boundary module owns why a transfer returned no receipt.

use std::error::Error;
use std::fmt;

/// Why a transfer returned no receipt. Partial output may have reached the
/// sink; none of these variants claims otherwise.
#[derive(Debug)]
pub enum TransferError<E> {
    /// The view refused the read before or during emission.
    Read(Box<dyn Error + 'static>),
    /// The sink refused a segment, an acknowledgement, or completion.
    Sink(E),
    /// The signal was raised before a segment.
    Cancelled {
        /// Segments applied before the signal was observed.
        segments: u64,
        /// Bytes applied before the signal was observed.
        bytes: u64,
    },
    /// The segment count or byte count is not representable.
    Accounting,
}

impl<E: fmt::Display> fmt::Display for TransferError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(source) => write!(formatter, "the view refused: {source}"),
            Self::Sink(source) => write!(formatter, "the sink refused: {source}"),
            Self::Cancelled { segments, bytes } => write!(
                formatter,
                "cancelled after {segments} segments and {bytes} bytes"
            ),
            Self::Accounting => formatter.write_str("the transfer's accounting overflowed"),
        }
    }
}

impl<E: Error + 'static> Error for TransferError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read(source) => Some(source.as_ref()),
            Self::Sink(source) => Some(source),
            Self::Cancelled { .. } | Self::Accounting => None,
        }
    }
}
