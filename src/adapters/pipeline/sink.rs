//! This boundary module owns the sink side: segments, the exactly-once
//! sink contract, and the sink over any `Write`.

use std::error::Error;
use std::fmt;
use std::io::{self, Write};

/// One verified segment: a borrowed slice of an immutable chunk, with its
/// position in the transfer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferSegment<'bytes> {
    index: u64,
    offset: u64,
    bytes: &'bytes [u8],
}

impl<'bytes> TransferSegment<'bytes> {
    pub(super) const fn new(index: u64, offset: u64, bytes: &'bytes [u8]) -> Self {
        Self {
            index,
            offset,
            bytes,
        }
    }

    /// The segment's ordinal in the transfer, from zero.
    #[must_use]
    pub const fn index(self) -> u64 {
        self.index
    }

    /// The logical offset of the first byte.
    #[must_use]
    pub const fn offset(self) -> u64 {
        self.offset
    }

    /// The verified bytes.
    #[must_use]
    pub const fn bytes(self) -> &'bytes [u8] {
        self.bytes
    }
}

/// Where a transfer applies its segments.
///
/// The transfer applies segments in index order, each exactly once,
/// acknowledges every window, and completes once after the last segment.
/// A sink refuses any other sequence.
pub trait TransferSink {
    /// The sink's refusal.
    type Error: Error + 'static;

    /// Applies one segment.
    ///
    /// # Errors
    ///
    /// Returns the sink's refusal; the transfer stops without a receipt.
    fn apply(&mut self, segment: TransferSegment<'_>) -> Result<(), Self::Error>;

    /// Makes every applied segment of the window durable to the sink's
    /// own standard.
    ///
    /// # Errors
    ///
    /// As [`Self::apply`].
    fn acknowledge(&mut self) -> Result<(), Self::Error>;

    /// Finishes the transfer after the last acknowledgement.
    ///
    /// # Errors
    ///
    /// As [`Self::apply`].
    fn complete(&mut self) -> Result<(), Self::Error>;
}

/// A sink over any writer, applying each segment exactly once in order and
/// flushing on every acknowledgement.
#[must_use]
#[derive(Debug)]
pub struct WriteSink<W> {
    inner: W,
    next_index: u64,
    offset: u64,
}

impl<W: Write> WriteSink<W> {
    /// Wraps `inner`, expecting the first segment next.
    pub const fn new(inner: W) -> Self {
        Self {
            inner,
            next_index: 0,
            offset: 0,
        }
    }

    /// The writer, with everything applied so far.
    pub fn into_inner(self) -> W {
        self.inner
    }

    /// Segments applied so far.
    #[must_use]
    pub const fn applied(&self) -> u64 {
        self.next_index
    }
}

impl<W: Write> TransferSink for WriteSink<W> {
    type Error = WriteSinkError;

    fn apply(&mut self, segment: TransferSegment<'_>) -> Result<(), WriteSinkError> {
        if segment.index() != self.next_index || segment.offset() != self.offset {
            return Err(WriteSinkError::OutOfOrder {
                expected_index: self.next_index,
                observed_index: segment.index(),
                expected_offset: self.offset,
                observed_offset: segment.offset(),
            });
        }
        let index = segment.index();
        self.inner
            .write_all(segment.bytes())
            .map_err(|source| WriteSinkError::Write { index, source })?;
        let length = u64::try_from(segment.bytes().len())
            .map_err(|_source| WriteSinkError::Accounting { index })?;
        self.offset = self
            .offset
            .checked_add(length)
            .ok_or(WriteSinkError::Accounting { index })?;
        self.next_index = self
            .next_index
            .checked_add(1)
            .ok_or(WriteSinkError::Accounting { index })?;
        Ok(())
    }

    fn acknowledge(&mut self) -> Result<(), WriteSinkError> {
        self.inner
            .flush()
            .map_err(|source| WriteSinkError::Flush { source })
    }

    fn complete(&mut self) -> Result<(), WriteSinkError> {
        self.acknowledge()
    }
}

/// Why a [`WriteSink`] refused.
#[derive(Debug)]
pub enum WriteSinkError {
    /// A segment arrived out of order or twice.
    OutOfOrder {
        /// The index the sink expected.
        expected_index: u64,
        /// The index that arrived.
        observed_index: u64,
        /// The offset the sink expected.
        expected_offset: u64,
        /// The offset that arrived.
        observed_offset: u64,
    },
    /// The writer refused the segment.
    Write {
        /// The segment being written.
        index: u64,
        /// The exact failure.
        source: io::Error,
    },
    /// The writer refused to flush.
    Flush {
        /// The exact failure.
        source: io::Error,
    },
    /// The segment count or offset is not representable.
    Accounting {
        /// The segment being accounted.
        index: u64,
    },
}

impl fmt::Display for WriteSinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfOrder {
                expected_index,
                observed_index,
                expected_offset,
                observed_offset,
            } => write!(
                formatter,
                "segment {observed_index} at {observed_offset} arrived where segment \
                 {expected_index} at {expected_offset} was expected"
            ),
            Self::Write { index, .. } => write!(formatter, "writing segment {index} failed"),
            Self::Flush { .. } => formatter.write_str("flushing the sink failed"),
            Self::Accounting { index } => {
                write!(formatter, "segment {index} overflows the sink's accounting")
            }
        }
    }
}

impl Error for WriteSinkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Write { source, .. } | Self::Flush { source } => Some(source),
            Self::OutOfOrder { .. } | Self::Accounting { .. } => None,
        }
    }
}
