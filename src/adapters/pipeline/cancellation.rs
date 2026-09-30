//! This boundary module owns cancellation: a signal the transfer consults
//! before every segment.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Whether a transfer should stop before its next segment.
pub trait CancellationSignal {
    /// True once the transfer must stop.
    fn is_cancelled(&self) -> bool;
}

/// A transfer that runs to completion or failure.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeverCancelled;

impl CancellationSignal for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

impl CancellationSignal for AtomicBool {
    fn is_cancelled(&self) -> bool {
        self.load(Ordering::Acquire)
    }
}

/// A shareable flag: clone it into whatever may cancel the transfer.
#[derive(Clone, Debug, Default)]
pub struct CancellationFlag(Arc<AtomicBool>);

impl CancellationFlag {
    /// A flag that is not yet raised.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raises the flag; the transfer stops before its next segment.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl CancellationSignal for CancellationFlag {
    fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }
}
