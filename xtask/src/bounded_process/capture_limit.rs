//! This module owns retained-byte limits for captured child streams.

/// Maximum retained bytes for one child standard-output and standard-error pair.
#[derive(Clone, Copy)]
pub(crate) struct CaptureLimits {
    stderr_bytes: usize,
    stdout_bytes: usize,
}

impl CaptureLimits {
    /// Creates exact independent limits for standard output and standard error.
    #[must_use]
    pub(crate) const fn new(stdout_bytes: usize, stderr_bytes: usize) -> Self {
        Self {
            stderr_bytes,
            stdout_bytes,
        }
    }

    /// Returns the standard-error retained-byte limit.
    pub(super) const fn stderr_bytes(self) -> usize {
        self.stderr_bytes
    }

    /// Returns the standard-output retained-byte limit.
    pub(super) const fn stdout_bytes(self) -> usize {
        self.stdout_bytes
    }
}
