//! This boundary module owns the transfer receipt.

use super::TransferWindow;

/// What one completed transfer established: the read receipt the view
/// returned, and the segments, bytes, and acknowledgements the sink took.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "the receipt records the authenticated read and the sink's accounting"]
pub struct TransferReceipt<R> {
    read: R,
    segments: u64,
    bytes: u64,
    acknowledgements: u64,
    window: TransferWindow,
}

impl<R: Copy> TransferReceipt<R> {
    pub(super) const fn new(
        read: R,
        segments: u64,
        bytes: u64,
        acknowledgements: u64,
        window: TransferWindow,
    ) -> Self {
        Self {
            read,
            segments,
            bytes,
            acknowledgements,
            window,
        }
    }

    /// The view's own receipt for the authenticated read.
    pub const fn read(&self) -> R {
        self.read
    }

    /// Segments applied to the sink.
    #[must_use]
    pub const fn segments(&self) -> u64 {
        self.segments
    }

    /// Bytes applied to the sink.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Acknowledgements the sink gave, one per full or final window.
    #[must_use]
    pub const fn acknowledgements(&self) -> u64 {
        self.acknowledgements
    }

    /// The window the transfer ran under.
    pub const fn window(&self) -> TransferWindow {
        self.window
    }
}
