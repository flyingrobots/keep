//! Registered storage-profile replay during reconstruction.

use crate::{AdmittedLayout, ChunkSpan, FastCdc, LayoutEntry, LayoutId, RegisteredStorageProfile};

use super::{ProfileBoundary, ReconstructionError};

pub(super) struct ProfileVerifier<'a> {
    detector: FastCdc,
    observation: BoundaryObservation<'a>,
}

impl<'a> ProfileVerifier<'a> {
    pub(super) fn new(
        layout_id: LayoutId,
        layout: &'a AdmittedLayout,
    ) -> Result<Self, ReconstructionError> {
        if layout.profile() != RegisteredStorageProfile::FAST_CDC_64K_V1 {
            return Err(ReconstructionError::ProfileVerifierUnavailable {
                layout: layout_id,
                profile: layout.profile().id(),
            });
        }
        Ok(Self {
            detector: FastCdc::new(),
            observation: BoundaryObservation::new(layout_id, layout.entries()),
        })
    }

    pub(super) fn feed(&mut self, bytes: &[u8]) -> Result<(), ReconstructionError> {
        let observation = &mut self.observation;
        self.detector
            .feed(bytes, |span| observation.observe(span))
            .map_err(|source| ReconstructionError::ProfileChunking {
                layout: observation.layout,
                source,
            })?;
        observation.check()
    }

    pub(super) fn finish(mut self) -> Result<(), ReconstructionError> {
        let final_span =
            self.detector
                .finish()
                .map_err(|source| ReconstructionError::ProfileChunking {
                    layout: self.observation.layout,
                    source,
                })?;
        if let Some(span) = final_span {
            self.observation.observe(span);
        }
        self.observation.finish()
    }
}

struct BoundaryObservation<'a> {
    layout: LayoutId,
    expected: &'a [LayoutEntry],
    next: usize,
    mismatch: Option<ReconstructionError>,
}

impl<'a> BoundaryObservation<'a> {
    const fn new(layout: LayoutId, expected: &'a [LayoutEntry]) -> Self {
        Self {
            layout,
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
        self.mismatch = Some(ReconstructionError::ProfileBoundaryMismatch {
            layout: self.layout,
            index: self.next,
            expected: expected.map(ProfileBoundary::from),
            observed: Some(ProfileBoundary::from(observed)),
        });
    }

    fn check(&mut self) -> Result<(), ReconstructionError> {
        self.mismatch.take().map_or(Ok(()), Err)
    }

    fn finish(mut self) -> Result<(), ReconstructionError> {
        self.check()?;
        if let Some(expected) = self.expected.get(self.next).copied() {
            return Err(ReconstructionError::ProfileBoundaryMismatch {
                layout: self.layout,
                index: self.next,
                expected: Some(ProfileBoundary::from(expected)),
                observed: None,
            });
        }
        Ok(())
    }
}
