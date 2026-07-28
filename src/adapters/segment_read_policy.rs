//! Explicit resource policy for complete immutable-segment admission.

use crate::LayoutEntryLimit;

use super::SegmentRecordLimit;

/// Caller-selected resource bounds for complete segment admission.
///
/// These limits do not alter canonical bytes, logical identities, or physical
/// segment digests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentReadPolicy {
    record_limit: SegmentRecordLimit,
    layout_entry_limit: LayoutEntryLimit,
}

impl SegmentReadPolicy {
    /// The version-1 protocol maxima.
    pub const MAXIMUM: Self = Self::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM);

    /// Constructs a policy from independently checked resource limits.
    #[must_use]
    pub const fn new(
        record_limit: SegmentRecordLimit,
        layout_entry_limit: LayoutEntryLimit,
    ) -> Self {
        Self {
            record_limit,
            layout_entry_limit,
        }
    }

    /// Returns the configured complete-record cap.
    #[must_use]
    pub const fn record_limit(self) -> SegmentRecordLimit {
        self.record_limit
    }

    /// Returns the configured nested flat-layout entry cap.
    #[must_use]
    pub const fn layout_entry_limit(self) -> LayoutEntryLimit {
        self.layout_entry_limit
    }
}
