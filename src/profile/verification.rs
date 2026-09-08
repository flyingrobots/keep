//! This module owns streaming replay of one admitted storage profile.

use crate::{AdmittedLayout, ChunkSpan, FastCdc, LayoutEntry, RegisteredStorageProfile};

use super::{ProfileBoundary, StorageProfileVerificationError};

/// Streaming verifier for the profile and boundaries bound by one layout.
#[allow(
    clippy::redundant_pub_crate,
    reason = "sibling adapters share profile replay without depending on each other"
)]
pub(crate) struct StorageProfileVerifier<'a> {
    detector: FastCdc,
    observation: BoundaryObservation<'a>,
}

impl<'a> StorageProfileVerifier<'a> {
    /// Starts replay for one already admitted layout.
    pub(crate) fn new(layout: &'a AdmittedLayout) -> Result<Self, StorageProfileVerificationError> {
        if layout.profile() != RegisteredStorageProfile::FAST_CDC_64K_V1 {
            return Err(StorageProfileVerificationError::Unsupported {
                profile: layout.profile().id(),
            });
        }
        Ok(Self {
            detector: FastCdc::new(),
            observation: BoundaryObservation::new(layout.entries()),
        })
    }

    /// Feeds the next exact logical byte span in layout order.
    pub(crate) fn feed(&mut self, bytes: &[u8]) -> Result<(), StorageProfileVerificationError> {
        let observation = &mut self.observation;
        self.detector
            .feed(bytes, |span| observation.observe(span))
            .map_err(|source| StorageProfileVerificationError::Chunking { source })?;
        observation.check()
    }

    /// Finishes replay and requires the exact admitted boundary sequence.
    pub(crate) fn finish(mut self) -> Result<(), StorageProfileVerificationError> {
        let final_span = self
            .detector
            .finish()
            .map_err(|source| StorageProfileVerificationError::Chunking { source })?;
        if let Some(span) = final_span {
            self.observation.observe(span);
        }
        self.observation.finish()
    }
}

struct BoundaryObservation<'a> {
    expected: &'a [LayoutEntry],
    next: usize,
    mismatch: Option<StorageProfileVerificationError>,
}

impl<'a> BoundaryObservation<'a> {
    const fn new(expected: &'a [LayoutEntry]) -> Self {
        Self {
            expected,
            next: 0,
            mismatch: None,
        }
    }

    fn observe(&mut self, span: ChunkSpan) {
        if self.mismatch.is_some() {
            return;
        }
        let observed = LayoutEntry::from(span);
        let expected = self.expected.get(self.next).copied();
        if expected == Some(observed)
            && let Some(accepted) = self.expected.get(..=self.next)
        {
            self.next = accepted.len();
            return;
        }
        self.mismatch = Some(StorageProfileVerificationError::BoundaryMismatch {
            index: self.next,
            expected: expected.map(ProfileBoundary::from),
            observed: Some(ProfileBoundary::from(observed)),
        });
    }

    fn check(&mut self) -> Result<(), StorageProfileVerificationError> {
        self.mismatch.take().map_or(Ok(()), Err)
    }

    fn finish(mut self) -> Result<(), StorageProfileVerificationError> {
        self.check()?;
        if let Some(expected) = self.expected.get(self.next).copied() {
            return Err(StorageProfileVerificationError::BoundaryMismatch {
                index: self.next,
                expected: Some(ProfileBoundary::from(expected)),
                observed: None,
            });
        }
        Ok(())
    }
}
