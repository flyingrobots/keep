//! This module owns the writable storage port for segment continuation.

use super::{
    OpenedReusableSegment, RecoverySegmentResumeRequest, RecoverySegmentResumeStorageError,
    SegmentStage,
};

/// Reopens one exact reusable segment prefix under exclusive writer authority.
///
/// The operation consumes the storage capability so the returned stage can
/// retain writer authority for its full writable lifetime. On success, the
/// stage must contain exactly the returned bytes and be positioned immediately
/// after them. The returned materialization must be bounded by the segment
/// protocol maximum and must have been revalidated against `request` after the
/// writable handle and canonical directory entry were both admitted.
pub trait RecoverySegmentResumeStorage: Sized {
    /// Exclusively owned writable stage that retains writer authority.
    type Stage: SegmentStage;

    /// Reopens and materializes the exact reusable prefix.
    ///
    /// # Errors
    ///
    /// Returns [`RecoverySegmentResumeStorageError`] when the stage is absent,
    /// differs from the request, cannot be opened safely, or cannot be
    /// materialized and positioned exactly.
    fn open_reusable(
        self,
        request: RecoverySegmentResumeRequest,
    ) -> Result<OpenedReusableSegment<Self::Stage>, RecoverySegmentResumeStorageError>;
}
