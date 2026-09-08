//! Reference-adapter mapping for domain-owned storage-profile replay.

use crate::profile::{StorageProfileVerificationError, StorageProfileVerifier};
use crate::{AdmittedLayout, LayoutId};

use super::ReconstructionError;

pub(super) struct ProfileVerifier<'a> {
    layout: LayoutId,
    verifier: StorageProfileVerifier<'a>,
}

impl<'a> ProfileVerifier<'a> {
    pub(super) fn new(
        layout: LayoutId,
        admitted: &'a AdmittedLayout,
    ) -> Result<Self, ReconstructionError> {
        let verifier =
            StorageProfileVerifier::new(admitted).map_err(|error| map_error(layout, error))?;
        Ok(Self { layout, verifier })
    }

    pub(super) fn feed(&mut self, bytes: &[u8]) -> Result<(), ReconstructionError> {
        self.verifier
            .feed(bytes)
            .map_err(|error| map_error(self.layout, error))
    }

    pub(super) fn finish(self) -> Result<(), ReconstructionError> {
        self.verifier
            .finish()
            .map_err(|error| map_error(self.layout, error))
    }
}

const fn map_error(
    layout: LayoutId,
    error: StorageProfileVerificationError,
) -> ReconstructionError {
    match error {
        StorageProfileVerificationError::Unsupported { profile } => {
            ReconstructionError::ProfileVerifierUnavailable { layout, profile }
        }
        StorageProfileVerificationError::Chunking { source } => {
            ReconstructionError::ProfileChunking { layout, source }
        }
        StorageProfileVerificationError::BoundaryMismatch {
            index,
            expected,
            observed,
        } => ReconstructionError::ProfileBoundaryMismatch {
            layout,
            index,
            expected,
            observed,
        },
    }
}
