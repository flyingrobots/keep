//! This module owns optional closed-segment publication selection.

use super::filesystem_publisher_authority::FilesystemPublisherAuthority;
use super::{AdmittedSegment, ClosedSegment, SegmentPublicationError};

/// Segment-pool work required before publishing one catalog generation.
///
/// The selected form can be created only by consuming a handle-free
/// [`ClosedSegment`] receipt and binding it to the exact admitted stage bytes.
/// Storage adapters may require additional private provenance before they
/// accept that selection.
#[must_use]
pub struct SegmentPublication<'selection, 'records> {
    selected: Option<SelectedSegment<'selection, 'records>>,
}

struct SelectedSegment<'selection, 'records> {
    _closed: ClosedSegment,
    admitted: &'selection AdmittedSegment<'records>,
    authority: Option<FilesystemPublisherAuthority>,
}

impl<'selection, 'records> SegmentPublication<'selection, 'records> {
    /// Selects no new segment stage because all catalog segments are durable.
    pub const fn none() -> Self {
        Self { selected: None }
    }

    /// Binds one storage-agnostic closed stage to its exact admitted bytes.
    ///
    /// This selection carries no filesystem-publisher authority. Use
    /// [`crate::FilesystemCatalogPublisher::select_segment`] for filesystem
    /// publication.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentPublicationError`] when record count, byte length, or
    /// physical digest disagrees.
    pub fn one(
        closed: ClosedSegment,
        admitted: &'selection AdmittedSegment<'records>,
    ) -> Result<Self, SegmentPublicationError> {
        Self::bind(closed, admitted, None)
    }

    pub(super) fn one_bound(
        closed: ClosedSegment,
        admitted: &'selection AdmittedSegment<'records>,
        authority: FilesystemPublisherAuthority,
    ) -> Result<Self, SegmentPublicationError> {
        Self::bind(closed, admitted, Some(authority))
    }

    fn bind(
        closed: ClosedSegment,
        admitted: &'selection AdmittedSegment<'records>,
        authority: Option<FilesystemPublisherAuthority>,
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
                authority,
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

    pub(super) fn is_bound_to(&self, authority: &FilesystemPublisherAuthority) -> bool {
        self.selected.as_ref().is_none_or(|selected| {
            selected
                .authority
                .as_ref()
                .is_some_and(|selected| selected.matches(authority))
        })
    }
}
