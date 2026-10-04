//! This module owns one registered, bounded retention realization policy.

use super::{RegisteredRetentionProfile, RetentionClosureLimits};

/// Registered realization semantics paired with caller-selected closure limits.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    profile: RegisteredRetentionProfile,
    limits: RetentionClosureLimits,
}

impl RetentionPolicy {
    /// Combines one registered profile with already-admitted closure limits.
    pub const fn new(profile: RegisteredRetentionProfile, limits: RetentionClosureLimits) -> Self {
        Self { profile, limits }
    }

    /// Returns the registered realization profile.
    pub const fn profile(self) -> RegisteredRetentionProfile {
        self.profile
    }

    /// Returns the admitted closure limits.
    pub const fn limits(self) -> RetentionClosureLimits {
        self.limits
    }
}
