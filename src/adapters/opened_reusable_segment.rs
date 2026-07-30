//! This module owns a storage-opened reusable segment materialization.

use super::SegmentStage;

/// Writable stage and exact bounded prefix returned by continuation storage.
///
/// The storage adapter constructing this value must prove that `stage`
/// contains exactly `encoded`, is positioned immediately after those bytes,
/// and retains exclusive writer authority for its lifetime. The recovery
/// executor independently re-admits `encoded` before returning the stage.
#[must_use]
pub struct OpenedReusableSegment<S>
where
    S: SegmentStage,
{
    stage: S,
    encoded: Box<[u8]>,
}

impl<S> OpenedReusableSegment<S>
where
    S: SegmentStage,
{
    /// Binds one storage-proven writable stage to its materialized prefix.
    pub const fn new(stage: S, encoded: Box<[u8]>) -> Self {
        Self { stage, encoded }
    }

    pub(super) fn into_parts(self) -> (S, Box<[u8]>) {
        (self.stage, self.encoded)
    }
}
