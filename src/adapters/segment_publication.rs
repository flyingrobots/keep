//! This module owns optional closed-segment publication selection.

use super::{AdmittedSegment, ClosedSegment, SegmentPublicationError};

/// Segment-pool work required before publishing one catalog generation.
///
/// The selected form can be created only by consuming a handle-free
/// [`ClosedSegment`] receipt and binding it to the exact admitted stage bytes.
#[must_use]
pub struct SegmentPublication<'selection, 'records> {
    selected: Option<SelectedSegment<'selection, 'records>>,
}

struct SelectedSegment<'selection, 'records> {
    _closed: ClosedSegment,
    admitted: &'selection AdmittedSegment<'records>,
}

impl<'selection, 'records> SegmentPublication<'selection, 'records> {
    /// Selects no new segment stage because all catalog segments are durable.
    pub const fn none() -> Self {
        Self { selected: None }
    }

    /// Binds one closed synchronized stage to its exact admitted bytes.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentPublicationError`] when record count, byte length, or
    /// physical digest disagrees.
    pub fn one(
        closed: ClosedSegment,
        admitted: &'selection AdmittedSegment<'records>,
    ) -> Result<Self, SegmentPublicationError> {
        let observed_length = u64::try_from(admitted.encoded().len()).map_err(|_source| {
            SegmentPublicationError::HostLength {
                observed: admitted.encoded().len(),
            }
        })?;
        if closed.record_count() != admitted.record_count() {
            return Err(SegmentPublicationError::RecordCount {
                expected: closed.record_count(),
                observed: admitted.record_count(),
            });
        }
        if closed.segment_length() != observed_length {
            return Err(SegmentPublicationError::SegmentLength {
                expected: closed.segment_length(),
                observed: observed_length,
            });
        }
        if closed.digest() != admitted.digest() {
            return Err(SegmentPublicationError::Digest {
                expected: closed.digest(),
                observed: admitted.digest(),
            });
        }
        Ok(Self {
            selected: Some(SelectedSegment {
                _closed: closed,
                admitted,
            }),
        })
    }

    pub(super) const fn admitted(&self) -> Option<&AdmittedSegment<'records>> {
        match &self.selected {
            Some(selected) => Some(selected.admitted),
            None => None,
        }
    }

    pub(super) fn into_admitted(self) -> Option<&'selection AdmittedSegment<'records>> {
        self.selected.map(|selected| selected.admitted)
    }
}
