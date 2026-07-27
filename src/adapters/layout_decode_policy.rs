//! Explicit bounded layout-decoding policy.

use crate::layout::{LayoutEntryLimit, LayoutId};

/// Caller-selected resource and identity requirements for layout decoding.
///
/// The entry cap bounds the only input-proportional allocation. An expected
/// identity is optional because a self-checksummed record can be admitted
/// before an external coordinate is available; supplying one strengthens the
/// final check without changing parsing or canonical bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutDecodePolicy {
    entry_limit: LayoutEntryLimit,
    expected_id: Option<LayoutId>,
}

impl LayoutDecodePolicy {
    /// Creates a bounded policy without an independently expected identity.
    #[must_use]
    pub const fn new(entry_limit: LayoutEntryLimit) -> Self {
        Self {
            entry_limit,
            expected_id: None,
        }
    }

    /// Requires the complete calculated record identity to match `expected`.
    #[must_use]
    pub const fn with_expected_id(mut self, expected: LayoutId) -> Self {
        self.expected_id = Some(expected);
        self
    }

    /// Returns the maximum entry count the decoder may materialize.
    #[must_use]
    pub const fn entry_limit(self) -> LayoutEntryLimit {
        self.entry_limit
    }

    /// Returns the independently expected identity, when configured.
    #[must_use]
    pub const fn expected_id(self) -> Option<LayoutId> {
        self.expected_id
    }
}
